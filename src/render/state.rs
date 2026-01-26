use std::{collections::HashMap, sync::Arc};

use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{game::world::WorldState, render::{assets::Assets, material::MaterialHandle, mesh::{CpuMesh, Mesh, MeshHandle}, model::{DrawModel, ModelHandle}, renderers::chunk::ChunkRenderer, scene::RenderScene, texture, uniforms::CameraUniform, vertex::Vertex}};

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
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 4
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 5
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 6
                }
            ]
        }
    }
}

pub struct PipelineLayouts {
    pub camera: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
}

impl PipelineLayouts {
    pub const CAMERA_SLOT: u32 = 0;
    pub const MATERIAL_SLOT: u32 = 1;

    pub fn new(device: &wgpu::Device) -> Self {
        let camera = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::CAMERA_SLOT,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ]
            }
        );

        let material = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Material Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    }
                ]
            }
        );

        Self {
            camera,
            material,
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
    chunk_renderer: ChunkRenderer,
    render_pipeline: wgpu::RenderPipeline,
    pipeline_layouts: PipelineLayouts,

    assets: Assets,

    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    instance_buffer: wgpu::Buffer,
    instance_count: u32,

    depth_texture: texture::Texture,
    block_material: MaterialHandle,
    // meshes: HashMap<MeshHandle, Mesh>,
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
    let shader_src = format!("{}\n{}", String::from(include_str!("../vertex.wgsl")), String::from(include_str!("../fragment.wgsl")));
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shaders"),
        source: wgpu::ShaderSource::Wgsl(shader_src.into())
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

    let pipeline_layouts = PipelineLayouts::new(&device);

    let camera_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &pipeline_layouts.camera,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: PipelineLayouts::CAMERA_SLOT,
                    resource: camera_buffer.as_entire_binding()
                }
            ],
        }
    );

    let mut assets = Assets::new(&device, &queue, &pipeline_layouts);

    let atlas = assets.load_texture("spritesheet_tiles.png", &device, &queue).await?;
    let block_material = assets.load_material(atlas, &device, &pipeline_layouts.material).await?;

    let depth_texture = texture::Texture::create_depth_texture(&device, &config, "Depth Texture");

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &pipeline_layouts.camera,
            &pipeline_layouts.material
        ],
        immediate_size: 0,
    });

    let chunk_renderer = ChunkRenderer::new(
        &device,
        &pipeline_layouts,
        config.format,
        texture::Texture::DEPTH_FORMAT,
        &shader_module
    );

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[
                Vertex::desc(),
                // Instance::desc(),
            ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default()
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
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
            pipeline_layouts,
            chunk_renderer,
            assets,

            camera_uniform,
            camera_buffer,
            camera_bind_group,

            instance_buffer,
            instance_count: 0,

            depth_texture,
            block_material,
            // meshes,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");
        }
    }

    pub fn render(&mut self, scene: RenderScene) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        let block_material = self.assets.get_material(self.block_material);
        for (coord, mesh) in scene.dirty_chunks {
            println!("Uploading chunk to GPU @ {:?}", coord);
            self.chunk_renderer.upload_chunk(&self.device, coord, mesh);
        }

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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None
            });

            self.chunk_renderer.draw(
                &mut render_pass,
                &self.camera_bind_group,
                &block_material.bind_group
            );
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
    }
}