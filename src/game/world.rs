use std::collections::HashMap;

use rand::Rng;
use winit::keyboard::KeyCode;

use crate::{game::{chunk::{Block, Chunk, ChunkCoord}, meshing::ChunkMesher}, input::Input, render::scene::RenderScene};

pub struct WorldState {
    pub time: f32,
    pub camera: CameraState,
    pub chunks: HashMap<ChunkCoord, Chunk>,
    rng: rand::rngs::ThreadRng,
}

impl WorldState {
    pub fn new() -> Self {
        let camera = CameraState {
            position: (8.0, 5.0, 25.0).into(),
            yaw: -std::f32::consts::FRAC_PI_2, // -90 degrees, points towards -Z
            pitch: -0.2,
            fov_y_radians: std::f32::consts::FRAC_PI_2,
            aspect: 800.0 / 600.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let chunks = HashMap::from([
            ((0, 0, 0), Chunk::test_chunk()),
            ((0, 0, 1), Chunk::test_chunk())
        ]);

        let rng = rand::rng();

        Self {
            time: 0.0,
            camera,
            chunks,
            rng,
        }
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        self.time += dt;

        self.handle_input(dt, input);

        let chunk = self.chunks.get_mut(&(0, 0, 0)).unwrap();

        let (x, y, z) = (self.rng.random_range(0..Chunk::CHUNK_SIZE), self.rng.random_range(0..Chunk::CHUNK_HEIGHT), self.rng.random_range(0..Chunk::CHUNK_SIZE));

        chunk.set_block((x, y, z), Block::Water);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.aspect = width as f32 / height as f32;
    }

    fn handle_input(&mut self, dt: f32, input: &Input) {
        const CAMERA_SENS: f32 = 0.001;
        const CAMERA_SPEED: f32 = 10.0;

        self.camera.yaw += input.mouse_delta.0 * CAMERA_SENS;
        self.camera.pitch -= input.mouse_delta.1 * CAMERA_SENS;

        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
        self.camera.pitch = self.camera.pitch.clamp(-max_pitch, max_pitch);


        if input.is_pressed(KeyCode::KeyW) {
            self.camera.position.z -= CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::KeyS) {
            self.camera.position.z += CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::KeyA) {
            self.camera.position.x -= CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::KeyD) {
            self.camera.position.x += CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::Space) {
            self.camera.position.y += CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::ControlLeft) {
            self.camera.position.y -= CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::KeyR) {
            let new_chunk = Chunk::test_chunk();
            self.chunks.insert((0, 0, 0), new_chunk);
        }
    }

    pub fn collect_render_data(&mut self) -> RenderScene {
        let mut render_scene = RenderScene {
            dirty_chunks: Vec::new()
        };

        for (coord, chunk) in self.chunks.iter_mut() {
            if chunk.is_dirty() {
                let mesh = ChunkMesher::create_mesh(chunk, *coord);
                chunk.mark_clean();
                render_scene.dirty_chunks.push((*coord, mesh))
            }
        }

        render_scene
    }
}

pub struct CameraState {
    pub position: glam::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_y_radians: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl CameraState {
    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let forward = glam::Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );

        let view = glam::Mat4::look_at_rh(
            self.position,
            self.position + forward,
            glam::Vec3::Y,
        );

        let proj = glam::Mat4::perspective_rh(
            self.fov_y_radians,
            self.aspect,
            self.znear,
            self.zfar
        );

        proj * view
    }

    pub fn build_isometric_view_projection_matrix(&self) -> glam::Mat4 {
        let yaw = 45.0_f32.to_radians();
        let pitch = -35.264_f32.to_radians();

        let forward = glam::Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );

        let view = glam::Mat4::look_at_rh(
            self.position,
            self.position + forward,
            glam::Vec3::Y,
        );

        let ortho_height = 20.0;
        let ortho_width = ortho_height * self.aspect;

        let proj = glam::Mat4::orthographic_rh(
            -ortho_width,
            ortho_width,
            -ortho_height,
            ortho_height,
            self.znear,
            self.zfar,
        );

        proj * view
    }
}