use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct SceneNode {
    pub(crate) name: Option<String>,
    pub(crate) children: Vec<usize>,
    primitives: Vec<usize>,
    pub(crate) visible: Option<bool>,
}

impl SceneNode {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            children: vec![],
            primitives: vec![],
            visible: None,
        }
    }
}

impl SceneNode {
    pub fn add_primitive(&mut self, primitive_index: usize) {
        self.primitives.push(primitive_index);
        if self.visible.is_none() {
            self.visible = Some(true);
        }
    }
}

#[derive(Default)]
pub(crate) struct Scene {
    pub(crate) scene_nodes: Vec<SceneNode>,
    pub(crate) root_indices: Vec<usize>,
}

impl Scene {
    pub(crate) fn at_least_one_visible(&self) -> bool {
        for node in &self.scene_nodes {
            if let Some(true) = node.visible {
                return true;
            }
        }

        false
    }

    pub(crate) fn set_all_visible(&mut self, visible: bool) {
        for node in &mut self.scene_nodes {
            if node.visible.is_some() {
                node.visible = Some(visible);
            }
        }
    }

    pub(crate) fn get_invisible_primitives(&self) -> HashSet<usize> {
        let mut invisible = HashSet::new();

        for root_index in self.root_indices.iter() {
            let node = &self.scene_nodes[*root_index];

            if Some(false) == node.visible {
                for primitive in &node.primitives {
                    invisible.insert(*primitive);
                }
            }

            for child in &node.children {
                self.insert_invisible_children(*child, &mut invisible);
            }
        }

        invisible
    }

    fn insert_invisible_children(&self, index: usize, invisible: &mut HashSet<usize>) {
        let node = &self.scene_nodes[index];

        if Some(false) == node.visible {
            for primitive in &node.primitives {
                invisible.insert(*primitive);
            }
        }

        if node.children.is_empty() {
            return;
        }

        for child in &node.children {
            self.insert_invisible_children(*child, invisible);
        }
    }
}
