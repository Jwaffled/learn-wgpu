#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    // 0-4 = x
    // 5-9 = y
    // 10-14 = z
    // 15-17 = faceId
    // 18-31 = texture index
    pub data: u32,
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Uint32,
                    shader_location: 0,
                },
            ]
        }
    }

    #[inline]
    pub fn pack_vertex(
        x: isize,
        y: isize,
        z: isize,
        corner_id: QuadCorner,
        face_id: VoxelFace,
        texture_index: u32,
    ) -> u32 {
        // 5 bits each, truncate if larger (shouldn't be)
        let x = x as u32 & 0b11111;
        let y = y as u32 & 0b11111;
        let z = z as u32 & 0b11111;

        // 3 bits
        let face = (face_id as u32) & 0b111;
        // 2 bits
        let corner = (corner_id as u32) & 0b11;
        
        // Remaining 32 - 5 - 3 - 2 = 22 bits
        let tex = texture_index & 0b11_1111_1111_1111_1111_1111;

        x
            | (y << 5)
            | (z << 10)
            | (face << 15)
            | (corner << 18)
            | (tex << 20)
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum QuadCorner {
    BottomLeft = 0,
    BottomRight = 1,
    TopLeft = 2,
    TopRight = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum VoxelFace {
    Top = 0,
    Bottom = 1,
    North = 2,
    South = 3,
    East = 4,
    West = 5,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DebugVertex {
    pub position: [f32; 3],
    pub color: [f32; 3]
}

impl DebugVertex {
        pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DebugVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 1,
                },
            ]
        }
    }
}