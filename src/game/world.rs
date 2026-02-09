use std::collections::{HashMap, HashSet};

use rand::Rng;
use winit::keyboard::KeyCode;

use crate::{game::{chunk::{Block, Chunk, ChunkCoord, LocalCoord, WorldCoord}, generator::WorldGenerator, meshing::{ChunkManager, ChunkMesher}, player::Player}, input::Input, render::{mesh::CpuMesh, scene::RenderScene}};

pub struct WorldState {
    pub time: f32,
    pub camera: CameraState,
    chunk_manager: ChunkManager,
    player: Player,
    frame_event: FrameEvent,

    // Configuration
    debug_enabled: bool,
}

#[derive(Default)]
pub struct FrameEvent {
    pub chunk_events: Vec<ChunkEvent>,
}

pub enum ChunkEvent {
    ChunkLoaded { coord: ChunkCoord, mesh: CpuMesh },
    ChunkUnloaded { coord: ChunkCoord },
    ChunkModified { coord: ChunkCoord, mesh: CpuMesh },
}

impl WorldState {
    const RENDER_DISTANCE: isize = 4;
    pub fn new() -> Self {
        let camera = CameraState {
            position: (8.0, 5.0, 25.0).into(),
            yaw: -std::f32::consts::FRAC_PI_2, // -90 degrees, points towards -Z
            pitch: -0.2,
            fov_y_radians: std::f32::consts::FRAC_PI_2,
            aspect: 800.0 / 600.0,
            znear: 0.1,
            zfar: 1000.0,
        };

        let chunk_manager = ChunkManager::new();

        let player = Player::new();

        Self {
            time: 0.0,
            camera,
            chunk_manager,
            player,
            frame_event: FrameEvent { chunk_events: Vec::new() },

            debug_enabled: false,
        }
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        self.time += dt;
        self.update_loaded_chunks();
        self.chunk_manager.update(&mut self.frame_event);

        let player_pos = self.player.position;

        self.handle_input(dt, input);

        if self.is_solid_at(self.player.position) {
            self.player.position = player_pos;
        }

    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.aspect = width as f32 / height as f32;
    }

    fn update_loaded_chunks(&mut self) {
        let player_pos = self.camera.position;
        let center = WorldCoord::from((player_pos.x as isize, player_pos.y as isize, player_pos.z as isize));
        let (chunk_coord, _) = center.to_chunk();

        let mut desired = HashSet::new();

        for dx in -Self::RENDER_DISTANCE..Self::RENDER_DISTANCE {
            for dz in -Self::RENDER_DISTANCE..Self::RENDER_DISTANCE {
                desired.insert(ChunkCoord::from((chunk_coord.x + dx, 0, chunk_coord.z + dz)));
            }
        }

        for coord in desired.iter() {
            self.chunk_manager.request_chunk(*coord);
        }

        self.chunk_manager.retain_chunks(desired, &mut self.frame_event);
    }

    fn handle_input(&mut self, dt: f32, input: &Input) {
        const CAMERA_SENS: f32 = 0.001;
        const CAMERA_SPEED: f32 = 100.0;
        const PLAYER_SPEED: f32 = 3.0;

        self.camera.yaw += input.mouse_delta.0 * CAMERA_SENS;
        self.camera.pitch -= input.mouse_delta.1 * CAMERA_SENS;

        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
        self.camera.pitch = self.camera.pitch.clamp(-max_pitch, max_pitch);


        if input.is_pressed(KeyCode::KeyW) {
            self.player.position.z -= PLAYER_SPEED * dt;
        }

        if input.is_pressed(KeyCode::KeyS) {
            self.player.position.z += PLAYER_SPEED * dt;
        }

        if input.is_pressed(KeyCode::KeyA) {
            self.player.position.x -= PLAYER_SPEED * dt;
        }

        if input.is_pressed(KeyCode::KeyD) {
            self.player.position.x += PLAYER_SPEED * dt;
        }

        if input.is_pressed(KeyCode::Space) {
            self.player.position.y += PLAYER_SPEED * dt;
        }

        if input.is_pressed(KeyCode::ControlLeft) {
            self.player.position.y -= PLAYER_SPEED * dt;
        }

        if input.is_pressed(KeyCode::ArrowUp) {
            self.camera.position.z -= CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::ArrowDown) {
            self.camera.position.z += CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::ArrowLeft) {
            self.camera.position.x -= CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::ArrowRight) {
            self.camera.position.x += CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::Enter) {
            self.camera.position.y += CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::ShiftRight) {
            self.camera.position.y -= CAMERA_SPEED * dt;
        }

        if input.is_pressed(KeyCode::F3) {
            self.debug_enabled = !self.debug_enabled;
        }
    }

    fn get_block(&self, coord: WorldCoord) -> Option<Block> {
        let (chunk_coord, local_coord) = coord.to_chunk();

        // let chunk = match self.chunk_manager.get_chunk(&chunk_coord) {
        //     Some(chunk) => chunk,
        //     None => return None,
        // };

        // return Some(chunk.get_block(local_coord));
        return None;
    }

    fn is_solid_at(&self, world_pos: glam::Vec3) -> bool {
        match self.get_block(WorldCoord::from((world_pos.x as isize, world_pos.y as isize, world_pos.z as isize))).unwrap_or_default() {
            Block::Air => false,
            other => true
        }

    }

    pub fn poll_events(&mut self) -> FrameEvent {
        std::mem::take(&mut self.frame_event)
    }

    pub fn generate_chunks(&mut self) {
        for x in -Self::RENDER_DISTANCE..Self::RENDER_DISTANCE {
            for z in -Self::RENDER_DISTANCE..Self::RENDER_DISTANCE {
                let coord = ChunkCoord { x, y: 0, z };
                self.chunk_manager.request_chunk(coord);
            }
        }
    }

    pub fn is_debug_enabled(&self) -> bool {
        return self.debug_enabled;
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