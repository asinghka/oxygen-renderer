use crate::vertex::Vertex;

pub(crate) fn load(path: &str) -> (Vec<Vertex>, Vec<u32>) {
    let (document, buffers, _) = gltf::import(path).expect("Failed to load glTF file");

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for node in document.nodes() {
        let transform_mat = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
        let normal_mat = glam::Mat3::from_mat4(transform_mat);

        let Some(mesh) = node.mesh() else { continue };

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let base = vertices.len() as u32;

            let positions = reader.read_positions().expect("Failed to read positions");
            let normals = reader.read_normals().expect("Failed to read normals");

            for (p, n) in positions.zip(normals) {
                let p_world = (transform_mat * glam::Vec3::from_array(p).extend(1.0)).truncate();
                let n_world = normal_mat * glam::Vec3::from_array(n);

                vertices.push(Vertex {
                    position: p_world.to_array(),
                    normal: n_world.to_array(),
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
