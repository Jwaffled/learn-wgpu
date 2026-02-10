use glyphon::{Attrs, Cache, Color, FontSystem, Metrics, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use wgpu::MultisampleState;

pub struct DebugStats {
    pub frame: FrameStats,
    pub chunk: ChunkStats,
}

pub struct FrameStats {
    pub fps: f32,
    pub frame_ms: f32,
}

pub struct ChunkStats {
    pub loaded: usize,
    pub in_flight: usize,
    pub meshing: usize,
    pub avg_gen_us: f32,
    pub avg_mesh_us: f32,
}

pub struct DebugTextRenderer {
    font_system: FontSystem,
    cache: SwashCache,
    atlas: TextAtlas,
    renderer: TextRenderer,
    buffer: glyphon::Buffer,
}

impl DebugTextRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, cache: &glyphon::Cache, surface_format: wgpu::TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let renderer = TextRenderer::new(
            &mut atlas,
            device,
            MultisampleState::default(),
            None
        );

        let buffer = glyphon::Buffer::new(
            &mut font_system,
            Metrics::new(24.0, 30.0)
        );

        Self {
            font_system,
            cache: swash_cache,
            atlas,
            renderer,
            buffer
        }
    }

    pub fn update_text(
        &mut self,
        stats: DebugStats,
        width: u32,
        height: u32,
    ) {
        self.buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32)    
        );

        let text = format!(
            "FPS: {:.1}\nFrame: {:.2} ms\nChunks: {}\nMeshing: {}\nAvg Gen: {:.2}μs\nAvg Mesh: {:.2}μs",
            stats.frame.fps,
            stats.frame.frame_ms,
            stats.chunk.loaded,
            stats.chunk.in_flight,
            stats.chunk.avg_gen_us,
            stats.chunk.avg_mesh_us
        );

        self.buffer.set_text(
            &mut self.font_system,
            &text,
            &Attrs::new(),
            glyphon::Shaping::Advanced,
            None,
        );

        self.buffer.shape_until_scroll(&mut self.font_system, false);
    }

    pub fn draw<'a>(
        &'a mut self, 
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
        viewport: &glyphon::Viewport
    ) {
        self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            viewport,
            [TextArea {
                buffer: &self.buffer,
                left: 10.0,
                top: 10.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: 600,
                    bottom: 300,
                },
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }],
            &mut self.cache
        )
        .unwrap();
        self.renderer.render(&self.atlas, viewport, render_pass).unwrap();
    }
}