use crate::vertex::Vertex;

pub(crate) fn load(path: &str) -> (Vec<Vertex>, Vec<u32>) {
    let (document, buffers, _) = gltf::import(path).expect("Failed to load glTF file");

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for node in document.nodes() {
        if node.name() != Some("Dragon") {
            continue;
        }

        let transform = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
        let Some(mesh) = node.mesh() else { continue };

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let base = vertices.len() as u32;

            let positions = reader.read_positions().expect("Failed to read positions");
            for p in positions {
                let world = transform * glam::Vec4::new(p[0], p[1], p[2], 1.0);
                vertices.push(Vertex {
                    position: [world[0], world[1], world[2]],
                });
            }

            let read_indices = reader.read_indices().expect("Failed to read indices");
            for i in read_indices.into_u32() {
                indices.push(base + i);
            }
        }
    }

    (vertices, indices)
}
