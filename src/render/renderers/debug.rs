use std::collections::HashSet;

use wgpu::util::DeviceExt;

use crate::{game::chunk::{Chunk, ChunkCoord}, render::{state::PipelineLayouts, vertex::{DebugVertex}}};

pub struct DebugRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    chunk_coords: HashSet<ChunkCoord>,
}

impl DebugRenderer {
    pub fn new(
        device: &wgpu::Device,
        layouts: &PipelineLayouts,
        surface_format: wgpu::TextureFormat, 
        depth_format: wgpu::TextureFormat, 
        shader: &wgpu::ShaderModule
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Debug Pipeline Layout"),
                bind_group_layouts: &[
                    &layouts.camera,
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
                        DebugVertex::desc()
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
                    topology: wgpu::PrimitiveTopology::LineList,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth_format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            }
        );

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Debug Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let vertex_count = 0;

        let chunk_coords = HashSet::new();

        Self {
            pipeline,
            vertex_buffer,
            vertex_count,
            chunk_coords,
        }
    }

    pub fn rebuild_chunk_borders(
        &mut self,
        device: &wgpu::Device,
        chunks: impl Iterator<Item = ChunkCoord>
    ) {
        let mut vertices = Vec::new();

        for chunk in chunks {
            vertices.extend(Self::chunk_border_vertices(chunk));
        }

        self.vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Debug Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        self.vertex_count = vertices.len() as u32;
        println!("Rebuilt chunk borders with {} vertices", vertices.len());
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(PipelineLayouts::CAMERA_SLOT, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }

    fn chunk_border_vertices(coord: ChunkCoord) -> Vec<DebugVertex> {
        let (cx, cy, cz) = coord.as_tuple();

        let min = [
            (cx * Chunk::CHUNK_SIZE as isize) as f32,
            (cy * Chunk::CHUNK_HEIGHT as isize) as f32,
            (cz * Chunk::CHUNK_SIZE as isize) as f32,
        ];

        let max = [
            min[0] + Chunk::CHUNK_SIZE as f32,
            min[1] + Chunk::CHUNK_HEIGHT as f32,
            min[2] + Chunk::CHUNK_SIZE as f32,
        ];

        let c = [1.0, 0.0, 1.0];

        let corners = [
            [min[0], min[1], min[2]],
            [max[0], min[1], min[2]],
            [max[0], min[1], max[2]],
            [min[0], min[1], max[2]],
            [min[0], max[1], min[2]],
            [max[0], max[1], min[2]],
            [max[0], max[1], max[2]],
            [min[0], max[1], max[2]],
        ];

        let edges = [
            (0,1),(1,2),(2,3),(3,0),
            (4,5),(5,6),(6,7),(7,4),
            (0,4),(1,5),(2,6),(3,7),
        ];

        edges.iter().flat_map(|(a,b)| {
            [
                DebugVertex { position: corners[*a], color: c },
                DebugVertex { position: corners[*b], color: c },
            ]
        }).collect()
    }
}