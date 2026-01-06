use std::{collections::HashMap, sync::Arc};

use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{render::{mesh::{Mesh, MeshHandle}, uniforms::CameraUniform, vertex::Vertex}, world::WorldState};

pub const MAX_CUBES: usize = 1000;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
}

impl Instance {
    pub fn new(model: glam::Mat4) -> Self {
        Self {
            model: model.to_cols_array_2d()
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 3
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 4
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 5
                }
            ]
        }
    }
}

pub struct RenderState {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub window: Arc<Window>,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,

    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    instance_buffer: wgpu::Buffer,
    instance_count: u32,

    meshes: HashMap<MeshHandle, Mesh>,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let inner_size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        })
        .await?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        })
        .await?;

    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps.formats.iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: inner_size.width,
        height: inner_size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../vertex.wgsl").into())
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../fragment.wgsl").into())
    });

    let camera_uniform = CameraUniform::new();

    let camera_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

    let instance_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (MAX_CUBES * std::mem::size_of::<Instance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );
    

    let camera_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                }
            ],
        }
    );

    let camera_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                }
            ],
        }
    );

    let meshes = HashMap::from([
        (MeshHandle(0), Mesh::cube(&device))
    ]);

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &camera_bind_group_layout,
        ],
        immediate_size: 0,
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: Some("vs_main"),
            buffers: &[
                Vertex::desc(),
                Instance::desc(),
            ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default()
        }),
        // fragment: None,
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });

        Ok(Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            window,
            is_surface_configured: false,
            render_pipeline,

            camera_uniform,
            camera_buffer,
            camera_bind_group,

            instance_buffer,
            instance_count: 0,

            meshes,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self, state: &WorldState) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let mut batches: HashMap<MeshHandle, Vec<Instance>> = HashMap::new();

        for object in state.objects.iter() {
            batches
                .entry(object.mesh_id)
                .or_default()
                .push(Instance::new(object.to_model_matrix()));
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0
                        }),
                        store: wgpu::StoreOp::Store
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for (mesh_handle, instances) in batches {
                let mesh = &self.meshes[&mesh_handle];

                self.instance_count = instances.len() as u32;
                self.queue.write_buffer(
                    &self.instance_buffer,
                    0,
                    bytemuck::cast_slice(&instances)
                );

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh.index_count, 0, 0..self.instance_count);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update(&mut self, world_state: &WorldState) {
        self.camera_uniform.update(&world_state.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&self.camera_uniform)
        );

        let instances = world_state
            .objects
            .iter()
            .map(|obj| Instance::new(obj.to_model_matrix()))
            .collect::<Vec<_>>();

        self.instance_count = instances.len() as u32;

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances)
        );
    }
}