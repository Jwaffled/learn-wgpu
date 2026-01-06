use wgpu::util::DeviceExt;

use crate::render::vertex::Vertex;

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn cube(device: &wgpu::Device) -> Self {
        let vertices: &[Vertex] = &[
            // Front
            Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0] },

            // Back
            Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] },
        ];

        let indices: &[u32] = &[
            0, 1, 2, 2, 3, 0, // front
            4, 6, 5, 6, 4, 7, // back
            4, 5, 1, 1, 0, 4, // bottom
            3, 2, 6, 6, 7, 3, // top
            1, 5, 6, 6, 2, 1, // right
            4, 0, 3, 3, 7, 4, // left
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MeshHandle(pub u32);