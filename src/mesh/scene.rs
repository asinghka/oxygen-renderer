use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct SceneNode {
    pub(crate) name: Option<String>,
    pub(crate) children: Vec<usize>,
    pub(crate) primitives: Vec<usize>,
    pub(crate) visible: bool,
}

impl SceneNode {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            children: vec![],
            primitives: vec![],
            visible: true,
        }
    }
}

#[derive(Default)]
pub(crate) struct Scene {
    pub(crate) scene_nodes: Vec<SceneNode>,
    pub(crate) root_indices: Vec<usize>,
}

impl Scene {
    pub(crate) fn get_invisible_nodes(&self) -> HashSet<usize> {
        let mut invisible = HashSet::new();

        for root_index in self.root_indices.iter() {
            if !self.scene_nodes[*root_index].visible {
                invisible.insert(*root_index);
            }

            for child in &self.scene_nodes[*root_index].children {
                self.insert_invisible_children(*child, &mut invisible);
            }
        }

        invisible
    }

    fn insert_invisible_children(&self, index: usize, invisible: &mut HashSet<usize>) {
        let node = &self.scene_nodes[index];

        if !node.visible {
            invisible.insert(index);
        }

        if node.children.is_empty() {
            return;
        }

        for child in &node.children {
            self.insert_invisible_children(*child, invisible);
        }
    }
}
