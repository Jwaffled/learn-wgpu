use crate::world::CameraState;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update(&mut self, camera: &CameraState) {
        self.view_proj = camera
            .build_view_projection_matrix()
            .to_cols_array_2d();
    }
}
