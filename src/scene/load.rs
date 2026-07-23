use crate::scene::{Material, Model, ModelNode, Primitive, TextureData, Vertex};
use gltf::Node;
use gltf::buffer::Data;
use gltf::image::Format;
use std::collections::HashSet;

pub(crate) fn load(path: String) -> Model {
    let (document, buffers, images) = gltf::import(path).expect("Failed to load glTF file");

    let mut scene_nodes = Vec::with_capacity(document.nodes().count());
    let mut root_indices = Vec::new();

    let mut primitives = Vec::new();
    let mut primitive_index = 0;

    let roots = document.default_scene().expect("No scene found").nodes();

    for root in roots {
        let root_index = visit(
            root,
            &buffers,
            glam::Mat4::IDENTITY,
            &mut scene_nodes,
            &mut primitives,
            &mut primitive_index,
        );

        root_indices.push(root_index);
    }

    let materials = document
        .materials()
        .map(|m| {
            let normal_texture = m.normal_texture();

            Material {
                color: m.pbr_metallic_roughness().base_color_factor(),
                albedo_texture: m
                    .pbr_metallic_roughness()
                    .base_color_texture()
                    .map(|info| info.texture().source().index()),
                normal_texture: normal_texture.as_ref().map(|t| t.texture().source().index()),
                bump: normal_texture.as_ref().map(|nt| nt.scale()).unwrap_or(0.0),
            }
        })
        .collect();

    let mut linear_images = HashSet::new();
    for material in document.materials() {
        if let Some(t) = material.normal_texture() {
            linear_images.insert(t.texture().source().index());
        }
        if let Some(t) = material.occlusion_texture() {
            linear_images.insert(t.texture().source().index());
        }
        if let Some(t) = material.pbr_metallic_roughness().metallic_roughness_texture() {
            linear_images.insert(t.texture().source().index());
        }
    }

    let mut textures = Vec::with_capacity(images.len());
    for (index, image) in images.into_iter().enumerate() {
        let width = image.width;
        let height = image.height;
        let srgb = !linear_images.contains(&index);

        let texture = match image.format {
            Format::R8G8B8 => {
                Some(TextureData {
                    pixels: expand_rgb_to_rgba(image.pixels),
                    width,
                    height,
                    srgb,
                })
            }
            Format::R8G8B8A8 => {
                Some(TextureData {
                    pixels: image.pixels,
                    width,
                    height,
                    srgb,
                })
            }
            _ => None,
        };

        textures.push(texture);
    }

    Model {
        model_nodes: scene_nodes,
        root_indices,
        primitives,
        materials,
        textures,
    }
}

fn visit(
    node: Node,
    buffers: &Vec<Data>,
    parent_world_matrix: glam::Mat4,
    scene_nodes: &mut Vec<ModelNode>,
    primitives: &mut Vec<Primitive>,
    primitive_index: &mut usize,
) -> usize {
    let mut scene_node = ModelNode::new(node.name().map(|s| s.to_string()));

    let model = parent_world_matrix * glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            scene_node.add_primitive(*primitive_index);
            *primitive_index += 1;

            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions = reader.read_positions().expect("Failed to read positions");
            let normals = reader.read_normals().expect("Failed to read normals");

            let uvs = reader.read_tex_coords(0).map(|tex_coords| tex_coords.into_f32().collect::<Vec<_>>());
            let uvs = uvs.into_iter().flatten().chain(std::iter::repeat([0.0; 2]));

            let tangents = reader.read_tangents().map(|t| t.collect::<Vec<_>>());
            let tangents = tangents.into_iter().flatten().chain(std::iter::repeat([1.0, 0.0, 0.0, 1.0]));

            for (((position, normal), uv), tangent) in positions.zip(normals).zip(uvs).zip(tangents) {
                vertices.push(Vertex {
                    position,
                    normal,
                    uv,
                    tangent,
                });
            }

            let read_indices = reader.read_indices().expect("Failed to read indices");
            for i in read_indices.into_u32() {
                indices.push(i);
            }

            let material = primitive.material().index();

            primitives.push(Primitive {
                vertices,
                indices,
                model,
                material,
            })
        }
    }

    for child in node.children() {
        let child_index = visit(child, buffers, model, scene_nodes, primitives, primitive_index);
        scene_node.children.push(child_index);
    }

    scene_nodes.push(scene_node);
    scene_nodes.len() - 1
}

fn expand_rgb_to_rgba(pixels: Vec<u8>) -> Vec<u8> {
    pixels
        .chunks_exact(3)
        .flat_map(|pixels| [pixels[0], pixels[1], pixels[2], 255_u8])
        .collect()
}
