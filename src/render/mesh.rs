use wgpu::util::DeviceExt;

use crate::{game::chunk::{Block, Chunk}, render::{material::MaterialHandle, vertex::Vertex}};

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn cube(device: &wgpu::Device) -> Self {
        let vertices: &[Vertex] = &[
            Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.5, -0.5, 0.5], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.5, 0.5, -0.5], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.5, 0.5, 0.5], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, 0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5, 0.5, -0.5], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 0.0] },
        ];
        
        let indices: &[u32] = &[
            0, 1, 2, 2, 3, 0,       // Front
            4, 5, 6, 6, 7, 4,       // Back
            8, 9, 10, 10, 11, 8,    // Right
            12, 13, 14, 14, 15, 12, // Left
            16, 17, 18, 18, 19, 16, // Top
            20, 21, 22, 22, 23, 20, // Bottom
        ];

        Self::from_vertices_indices(vertices, indices, device)
    }

    pub fn from_vertices_indices(vertices: &[Vertex], indices: &[u32], device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
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

#[derive(Debug)]
pub struct CpuMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}