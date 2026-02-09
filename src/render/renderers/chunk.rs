use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::{game::chunk::ChunkCoord, render::{mesh::CpuMesh, state::PipelineLayouts, vertex::Vertex}};

pub struct ChunkGpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct ChunkRenderer {
    pipeline: wgpu::RenderPipeline,
    chunk_meshes: HashMap<ChunkCoord, ChunkGpuMesh>
}

impl ChunkRenderer {
    pub fn new(
        device: &wgpu::Device,
        layouts: &PipelineLayouts,
        surface_format: wgpu::TextureFormat, 
        depth_format: wgpu::TextureFormat, 
        shader: &wgpu::ShaderModule
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Chunk Pipeline Layout"),
                bind_group_layouts: &[
                    &layouts.camera,
                    &layouts.material
                ],
                immediate_size: 0
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Chunk Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        Vertex::desc()
                    ],
                    compilation_options: Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL
                    })],
                    compilation_options: Default::default()
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    cull_mode: Some(wgpu::Face::Back),
                    front_face: wgpu::FrontFace::Ccw,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth_format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            }
        );

        Self {
            pipeline,
            chunk_meshes: Default::default()
        }
    }

    pub fn load_chunk(
        &mut self,
        device: &wgpu::Device,
        chunk_coord: ChunkCoord,
        mesh: CpuMesh
    ) {
        println!("Num chunks loaded in renderer: {}", self.chunk_meshes.len());
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX
            }
        );

        self.chunk_meshes.insert(
            chunk_coord,
            ChunkGpuMesh {
                vertex_buffer,
                index_buffer,
                index_count: mesh.indices.len() as u32,
            }
        );
    }

    pub fn unload_chunk(&mut self, coord: ChunkCoord) {
        self.chunk_meshes.remove(&coord);
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        material_bind_group: &'a wgpu::BindGroup
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(PipelineLayouts::CAMERA_SLOT, camera_bind_group, &[]);
        render_pass.set_bind_group(PipelineLayouts::MATERIAL_SLOT, material_bind_group, &[]);

        for mesh in self.chunk_meshes.values() {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}