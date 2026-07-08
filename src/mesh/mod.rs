mod primitive;
mod vertex;

use gltf::Node;
use gltf::buffer::Data;
pub(crate) use primitive::*;
pub(crate) use vertex::*;

pub(crate) fn load(path: &str) -> (Vec<Primitive>, u32, u32) {
    let (document, buffers, _) = gltf::import(path).expect("Failed to load glTF file");

    let mut primitives = Vec::new();
    let mut num_vertices = 0;
    let mut num_indices = 0;

    let roots = document.default_scene().expect("No scene found").nodes();

    for root in roots {
        visit(root, &buffers, glam::Mat4::IDENTITY, &mut primitives, &mut num_vertices, &mut num_indices);
    }

    (primitives, num_vertices, num_indices)
}

fn visit(
    node: Node,
    buffers: &Vec<Data>,
    parent_world_matrix: glam::Mat4,
    primitives: &mut Vec<Primitive>,
    num_vertices: &mut u32,
    num_indices: &mut u32,
) {
    let model = parent_world_matrix * glam::Mat4::from_cols_array_2d(&node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
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

            *num_vertices += vertices.len() as u32;
            *num_indices += indices.len() as u32;

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
        visit(child, buffers, model, primitives, num_vertices, num_indices);
    }
}
