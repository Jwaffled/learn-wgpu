use std::sync::Arc;

use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::Window};

use crate::{render::state::RenderState, world::WorldState};

mod texture;
mod model;
mod resources;
mod world;
mod render;

pub struct App {
    world_state: WorldState,
    render_state: Option<RenderState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            world_state: WorldState::new(),
            render_state: None
        }
    }
}

impl ApplicationHandler<RenderState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.render_state = Some(pollster::block_on(RenderState::new(window)).unwrap());
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: RenderState) {
        self.render_state = Some(event);
    }

    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        let state = match &mut self.render_state {
            Some(canvas) => canvas,
            None => return
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
                self.world_state.resize(size.width, size.height);
            },
            WindowEvent::RedrawRequested => {
                self.world_state.update(0.01);
                state.update(&self.world_state);
                match state.render(&self.world_state) {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}