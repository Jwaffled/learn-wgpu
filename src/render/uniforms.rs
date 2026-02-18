use crate::game::world::CameraState;

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
            // .build_isometric_view_projection_matrix()
            .to_cols_array_2d();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkOffsetUniform {
    pub offset: [f32; 3],
    _pad: f32,
}

impl ChunkOffsetUniform {
    pub fn new(offset: [f32; 3]) -> Self {
        Self {
            offset,
            _pad: 0.0,
        }
    }
}