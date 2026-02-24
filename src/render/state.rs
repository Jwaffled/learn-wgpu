use std::{sync::Arc, time::Instant};

use glyphon::{Resolution, Viewport};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{game::{chunk::AABB, world::{CameraState, ChunkEvent, FrameEvent, RenderEvent, WorldState}}, render::{assets::Assets, material::MaterialHandle, mesh::{CpuMesh, Mesh, MeshHandle}, model::{DrawModel, ModelHandle}, renderers::{chunk::ChunkRenderer, debug::DebugRenderer, debug_text::{DebugStats, DebugTextRenderer, FrameStats}}, scene::RenderScene, texture, uniforms::CameraUniform, vertex::{DebugVertex, Vertex}}};

pub struct PipelineLayouts {
    pub camera: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
    pub chunk_offset: wgpu::BindGroupLayout,
}

impl PipelineLayouts {
    pub const CAMERA_SLOT: u32 = 0;
    pub const MATERIAL_SLOT: u32 = 1;
    pub const CHUNK_OFFSET_SLOT: u32 = 2;

    pub fn new(device: &wgpu::Device) -> Self {
        let camera = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
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
                            view_dimension: wgpu::TextureViewDimension::D2Array,
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

        let chunk_offset = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Chunk Offset Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        Self {
            camera,
            material,
            chunk_offset,
        }
    }
}

pub struct RenderState {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'static>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    viewport: glyphon::Viewport,
    pub window: Arc<Window>,
    is_surface_configured: bool,
    chunk_renderer: ChunkRenderer,
    debug_renderer: DebugRenderer,
    debug_text_renderer: DebugTextRenderer,
    pipeline_layouts: PipelineLayouts,

    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    frustum: Frustum,

    debug_camera: CameraState,
    debug_camera_uniform: CameraUniform,
    debug_camera_buffer: wgpu::Buffer,
    debug_camera_bind_group: wgpu::BindGroup,

    depth_texture: texture::Texture,
    chunk_material: Option<MaterialHandle>,

    // Stats
    last_frame_time: Instant,
    frame_ms: f32,
    fps: f32,
    
    fps_accumulator: f32,
    fps_frames: u32,

    draw_calls: u32,
    vertices: u32,
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
                required_features: 
                    wgpu::Features::TIMESTAMP_QUERY |
                    wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS |
                    wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES |
                    wgpu::Features::POLYGON_MODE_LINE,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let text_cache = glyphon::Cache::new(&device);

        let viewport = Viewport::new(&device, &text_cache);

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

        let debug_shader_src = include_str!("../debug_shader.wgsl");

        let debug_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Shader"),
            source: wgpu::ShaderSource::Wgsl(debug_shader_src.into())
        });

        let camera_uniform = CameraUniform::new();
        let frustum = camera_uniform.frustum();

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let debug_camera_uniform = CameraUniform::new();
        let debug_camera = CameraState { 
            position: (-200.0, 100.0, -200.0).into(),
            yaw: 0.0, // -90 degrees, points towards -Z
            pitch: -0.2,
            fov_y_radians: std::f32::consts::FRAC_PI_2,
            aspect: 800.0 / 600.0,
            znear: 0.1,
            zfar: 1000.0,
        };

        let debug_camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Debug Camera Buffer"),
                contents: bytemuck::cast_slice(&[debug_camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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

        let debug_camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Debug Camera Bind Group"),
                layout: &pipeline_layouts.camera,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: PipelineLayouts::CAMERA_SLOT,
                        resource: debug_camera_buffer.as_entire_binding(),
                    }
                ]
            }
        );

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "Depth Texture");

        let chunk_renderer = ChunkRenderer::new(
            &device,
            &pipeline_layouts,
            config.format,
            texture::Texture::DEPTH_FORMAT,
            &shader_module
        );

        let debug_renderer = DebugRenderer::new(
            &device,
            &pipeline_layouts,
            config.format,
            texture::Texture::DEPTH_FORMAT,
            &debug_shader
        );

        let debug_text_renderer = DebugTextRenderer::new(
            &device,
            &queue,
            &text_cache,
            config.format,
        );

        // Stats
        let last_frame_time = Instant::now();
        let frame_ms = 0.0;
        let fps = 0.0;
        let fps_accumulator = 0.0;
        let fps_frames = 0;

        Ok(Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            viewport,
            window,
            is_surface_configured: false,
            pipeline_layouts,
            chunk_renderer,
            debug_renderer,
            debug_text_renderer,

            camera_uniform,
            camera_buffer,
            camera_bind_group,
            frustum,

            debug_camera,
            debug_camera_uniform,
            debug_camera_buffer,
            debug_camera_bind_group,

            depth_texture,
            chunk_material: None,

            // Stats
            last_frame_time,
            frame_ms,
            fps,
            fps_accumulator,
            fps_frames,

            draw_calls: 0,
            vertices: 0,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");
            self.viewport.update(&self.queue, Resolution {
                width,
                height
            });
            self.debug_camera.aspect = width as f32 / height as f32;
        }
    }

    pub fn process(&mut self, event: FrameEvent) {
        for event in event.render_events {
            match event {
                RenderEvent::ChunkMaterialChanged { handle } => {
                    self.chunk_material = Some(handle);
                }
            }
        }

        for event in event.chunk_events {
            match event {
                ChunkEvent::ChunkMeshReady { coord, mesh } => {
                    self.vertices += mesh.vertices.len() as u32;
                    self.chunk_renderer.load_chunk(&self.device, &self.pipeline_layouts, coord, mesh);
                    self.debug_renderer.load_chunk(&self.device, coord);
                },
                ChunkEvent::ChunkUnloaded { coord } => {
                    let mesh = self.chunk_renderer.unload_chunk(coord);
                    if let Some(mesh) = mesh {
                        self.vertices -= mesh.vertex_count;
                    }
                    self.debug_renderer.unload_chunk(coord);
                },
                ChunkEvent::ChunkModified { coord, mesh } => self.chunk_renderer.load_chunk(&self.device, &self.pipeline_layouts, coord, mesh),
            }
        }
    }

    pub fn render(&mut self, world: &WorldState, assets: &Assets) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured || self.chunk_material.is_none() {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        let block_material = assets.get_material(self.chunk_material.unwrap());
        self.debug_text_renderer.update_text(
            DebugStats {
                chunk: world.chunk_stats(),
                frame: self.frame_stats(),
            },
            500,
            300
        );

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

            render_pass.set_viewport(
                0.0,
                0.0,
                self.config.width as f32,
                self.config.height as f32,
                0.0,
                1.0
            );

            self.draw_calls = self.chunk_renderer.draw(
                &mut render_pass,
                &self.camera_bind_group,
                &block_material.bind_group,
                &self.frustum
            );

            if world.debug_enabled {
                self.debug_renderer.draw(
                    &mut render_pass,
                    &self.camera_bind_group
                );
            }

            if world.debug_enabled {
                let pip_width = self.config.width as f32 * 0.3;
                let pip_height = self.config.height as f32 * 0.3;
                render_pass.set_viewport(
                    self.config.width as f32 - pip_width - 10.0,
                    10.0,
                    pip_width,
                    pip_height,
                    0.0,
                    1.0
                );

                self.chunk_renderer.draw(&mut render_pass, &self.debug_camera_bind_group, &block_material.bind_group, &self.frustum);
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Text Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    depth_slice: None,
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }
                })],
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
                depth_stencil_attachment: None,
            });

            self.debug_text_renderer.draw(&self.device, &self.queue, &mut render_pass, &self.viewport);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        let now = Instant::now();
        let delta = now - self.last_frame_time;
        self.last_frame_time = now;

        let frame_ms = delta.as_secs_f32() * 1000.0;
        self.frame_ms = frame_ms;

        self.fps_accumulator += delta.as_secs_f32();
        self.fps_frames += 1;

        if self.fps_accumulator >= 0.5 {
            self.fps = self.fps_frames as f32 / self.fps_accumulator;
            self.fps_accumulator = 0.0;
            self.fps_frames = 0;
        }

        Ok(())
    }

    pub fn update(&mut self, world_state: &WorldState) {
        self.camera_uniform.update(&world_state.camera);
        self.debug_camera_uniform.update(&self.debug_camera);
        self.frustum = self.camera_uniform.frustum();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&self.camera_uniform)
        );
        self.queue.write_buffer(
            &self.debug_camera_buffer,
            0,
            bytemuck::bytes_of(&self.debug_camera_uniform)
        );
    }

    pub fn frame_stats(&self) -> FrameStats {
        FrameStats { 
            fps: self.fps,
            frame_ms: self.frame_ms,
            draw_calls: self.draw_calls,
            vertices: self.vertices,
        }
    }
}

pub struct Plane {
    pub normal: glam::Vec3,
    pub d: f32,
}

impl Plane {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        let normal = glam::Vec3::new(x, y, z);
        let length = normal.length();

        Self {
            normal: normal / length,
            d: w / length
        }
    }
}

pub struct Frustum {
    pub planes: [Plane; 6]
}

impl Frustum {
    pub fn from_view_proj(m: [[f32; 4]; 4]) -> Self {
        let left = Plane::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0], m[3][3] + m[3][0]);
        let right = Plane::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0], m[3][3] - m[3][0]);
        let bottom = Plane::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1], m[3][3] + m[3][1]);
        let top = Plane::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1], m[3][3] - m[3][1]);
        let near = Plane::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2], m[3][3] + m[3][2]);
        let far = Plane::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2], m[3][3] - m[3][2]);

        let planes = [
            left, right, bottom, top, near, far
        ];

        Self { planes }
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            let p = glam::Vec3::new(
                if plane.normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
                if plane.normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
                if plane.normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
            );

            if plane.normal.dot(p) + plane.d < 0.0 {
                return false;
            }
        }

        true
    }
}