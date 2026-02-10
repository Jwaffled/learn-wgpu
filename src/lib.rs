use std::sync::Arc;

use libnoise::Source;
use libnoise::prelude::*;
use winit::{application::ApplicationHandler, event::{DeviceEvent, Event, KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use crate::{game::world::WorldState, input::Input, render::state::RenderState};

mod game;
mod render;
mod input;

pub struct App {
    world_state: WorldState,
    render_state: Option<RenderState>,
    window: Option<Arc<Window>>,
    cursor_visible: bool,
    input_state: Input,
}

impl App {
    pub fn new() -> Self {
        // let noise = Source::perlin(42).scale([0.1; 2]);
        // Visualizer::<2>::new([100, 100], &noise)
        //     .write_to_file("image-test.png")
        //     .unwrap();
        let mut world_state = WorldState::new();
        world_state.generate_chunks();
        Self {
            world_state,
            render_state: None,
            window: None,
            cursor_visible: true,
            input_state: Input::new()
        }
    }
}

impl ApplicationHandler<RenderState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        window.set_cursor_visible(false);
        window.set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
            .unwrap();
        self.window = Some(window.clone());
        self.cursor_visible = false;
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

        let window = match &self.window {
            Some(window) => window,
            None => return
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
                self.world_state.resize(size.width, size.height);
            },
            WindowEvent::RedrawRequested => {
                self.world_state.update(0.01, &self.input_state);
                state.update(&self.world_state);
                let events = self.world_state.poll_events();
                state.process(events);
                // let render_scene = self.world_state.collect_render_data();
                // let debug_enabled = self.world_state.is_debug_enabled();
                self.input_state.mouse_delta = (0.0, 0.0);
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
            WindowEvent::KeyboardInput { 
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                    ..
             } => {
                if code == KeyCode::Escape && key_state.is_pressed() {
                    window.set_cursor_visible(!self.cursor_visible);
                    let cursor_grab = if self.cursor_visible {
                        winit::window::CursorGrabMode::Locked
                    } else {
                        winit::window::CursorGrabMode::None
                    };
                    self.cursor_visible = !self.cursor_visible;
                    window.set_cursor_grab(cursor_grab).unwrap();
                    return;
                }

                if key_state.is_pressed() {
                    self.input_state.key_down(code);
                } else {
                    self.input_state.key_up(code);
                }
             },
             
            _ => {}
        }
    }

    fn device_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            device_id: winit::event::DeviceId,
            event: DeviceEvent,
        ) {
        let state = match &mut self.render_state {
            Some(canvas) => canvas,
            None => return
        };

        match event {
            DeviceEvent::MouseMotion { delta } => {
                let (dx, dy) = delta;
                self.input_state.mouse_delta.0 += dx as f32;
                self.input_state.mouse_delta.1 += dy as f32;
            },
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