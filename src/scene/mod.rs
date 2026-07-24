mod light;
mod load;
mod material;
mod model;
mod primitive;
mod texture;
mod vertex;

use crate::camera::Camera;
pub(crate) use light::*;
pub(crate) use load::*;
pub(crate) use material::*;
pub(crate) use model::*;
pub(crate) use primitive::*;
pub(crate) use texture::*;
pub(crate) use vertex::*;

#[derive(Default)]
pub(crate) struct Scene {
    pub(crate) camera: Camera,
    pub(crate) light: Light,
    pub(crate) model: Model,
}
