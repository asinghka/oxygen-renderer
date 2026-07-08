use std::mem::offset_of;

// Ensure uniform values are 16-byte-aligned (std140)
const _: () = assert!(size_of::<CameraUniform>() == 80);
const _: () = assert!(offset_of!(CameraUniform, view_projection_matrix) == 16);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    pub(crate) eye: [f32; 3],
    pub(crate) pad: u32,
    pub(crate) view_projection_matrix: [[f32; 4]; 4],
}
