use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::{game::chunk::{Chunk, ChunkCoord}, render::{mesh::CpuMesh, state::PipelineLayouts, uniforms::ChunkOffsetUniform, vertex::Vertex}};

pub struct ChunkGpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub chunk_offset_buffer: wgpu::Buffer,
    pub chunk_offset: wgpu::BindGroup,
    pub index_count: u32,
    pub vertex_count: u32,
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
                    &layouts.material,
                    &layouts.chunk_offset,
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
                    polygon_mode: wgpu::PolygonMode::Fill,
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
        pipeline_layouts: &PipelineLayouts,
        chunk_coord: ChunkCoord,
        mesh: CpuMesh
    ) {
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

        let chunk_offset_uniform = ChunkOffsetUniform::new([chunk_coord.x as f32 * Chunk::CHUNK_SIZE as f32, chunk_coord.y as f32 * Chunk::CHUNK_SIZE as f32, chunk_coord.z as f32 * Chunk::CHUNK_SIZE as f32]);

        let chunk_offset_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Offset Buffer"),
                contents: bytemuck::cast_slice(&[chunk_offset_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let chunk_offset = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Chunk Layout Uniform"),
                layout: &pipeline_layouts.chunk_offset,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: chunk_offset_buffer.as_entire_binding(),
                    }
                ]
            }
        );

        self.chunk_meshes.insert(
            chunk_coord,
            ChunkGpuMesh {
                vertex_buffer,
                index_buffer,
                chunk_offset_buffer,
                chunk_offset,
                index_count: mesh.indices.len() as u32,
                vertex_count: mesh.vertices.len() as u32,
            }
        );
    }

    pub fn unload_chunk(&mut self, coord: ChunkCoord) -> Option<ChunkGpuMesh> {
        self.chunk_meshes.remove(&coord)
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        material_bind_group: &'a wgpu::BindGroup
    ) -> u32 {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(PipelineLayouts::CAMERA_SLOT, camera_bind_group, &[]);
        render_pass.set_bind_group(PipelineLayouts::MATERIAL_SLOT, material_bind_group, &[]);

        let mut draw_calls = 0;
        for mesh in self.chunk_meshes.values() {
            render_pass.set_bind_group(PipelineLayouts::CHUNK_OFFSET_SLOT, &mesh.chunk_offset, &[]);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            draw_calls += 1;
        }

        draw_calls
    }
}