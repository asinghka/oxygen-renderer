mod primitive;
mod scene;
mod vertex;

use gltf::Node;
use gltf::buffer::Data;
pub(crate) use primitive::*;
pub(crate) use scene::*;
pub(crate) use vertex::*;

pub(crate) fn load(path: &str) -> Scene {
    let (document, buffers, _) = gltf::import(path).expect("Failed to load glTF file");

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

    Scene {
        scene_nodes,
        root_indices,
        primitives,
    }
}

fn visit(
    node: Node,
    buffers: &Vec<Data>,
    parent_world_matrix: glam::Mat4,
    scene_nodes: &mut Vec<SceneNode>,
    primitives: &mut Vec<Primitive>,
    primitive_index: &mut usize,
) -> usize {
    let mut scene_node = SceneNode::new(node.name().map(|s| s.to_string()));

    let model = parent_world_matrix * glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            scene_node.add_primitive(*primitive_index);
            *primitive_index += 1;

            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions = reader.read_positions().expect("Failed to read positions");
            let normals = reader.read_normals().expect("Failed to read normals");

            for (position, normal) in positions.zip(normals) {
                vertices.push(Vertex { position, normal });
            }

            let read_indices = reader.read_indices().expect("Failed to read indices");
            for i in read_indices.into_u32() {
                indices.push(i);
            }

            let color = primitive.material().pbr_metallic_roughness().base_color_factor();

            primitives.push(Primitive {
                vertices,
                indices,
                model,
                color,
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
