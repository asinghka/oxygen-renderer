mod light;
mod load;
mod model;
mod primitive;
mod texture;
mod vertex;

use crate::camera::Camera;
pub(crate) use light::*;
pub(crate) use load::*;
pub(crate) use model::*;
pub(crate) use primitive::*;
pub(crate) use texture::*;
pub(crate) use vertex::*;

pub(crate) struct Scene {
    camera: Camera,
    light: Light,
    model: Model,
}
