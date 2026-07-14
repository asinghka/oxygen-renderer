mod load;
mod primitive;
mod texture;
mod vertex;

pub(crate) use load::*;
pub(crate) use primitive::*;
pub(crate) use texture::*;
pub(crate) use vertex::*;

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
    pub(crate) primitives: Vec<Primitive>,
    pub(crate) root_indices: Vec<usize>,
    pub(crate) textures: Vec<Option<TextureData>>,
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

        for node in &self.scene_nodes {
            if let Some(false) = node.visible {
                for primitive_index in &node.primitives {
                    invisible.insert(*primitive_index);
                }
            }
        }

        invisible
    }

    pub(crate) fn get_visible_primitive_stats(&self) -> (u32, u32) {
        let mut num_vertices = 0;
        let mut num_indices = 0;

        for node in &self.scene_nodes {
            if Some(true) != node.visible {
                continue;
            }

            for primitive_index in &node.primitives {
                let primitive = &self.primitives[*primitive_index];
                num_vertices += primitive.vertices.len() as u32;
                num_indices += primitive.indices.len() as u32;
            }
        }

        (num_vertices, num_indices)
    }
}
