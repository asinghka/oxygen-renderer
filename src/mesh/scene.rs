#[derive(Debug)]
pub(crate) struct SceneNode {
    pub(crate) name: Option<String>,
    pub(crate) children: Vec<usize>,
    pub(crate) primitives: Vec<usize>,
}

#[derive(Default)]
pub(crate) struct Scene {
    pub(crate) scene_nodes: Vec<SceneNode>,
    pub(crate) root_indices: Vec<usize>,
}
