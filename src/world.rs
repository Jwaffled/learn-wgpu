use crate::render::mesh::MeshHandle;

pub struct WorldState {
    pub time: f32,
    pub camera: CameraState,
    pub objects: Vec<ObjectState>
}

impl WorldState {
    pub fn new() -> Self {
        let camera = CameraState {
            position: (0.0, 1.0, 2.5).into(),
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.3,
            fov_y_radians: std::f32::consts::FRAC_PI_3,
            aspect: 800.0 / 600.0,
            znear: 0.1,
            zfar: 100.0,
        };

        Self {
            time: 0.0,
            camera,
            objects: vec![
                ObjectState {
                    position: (0.0, 0.0, 0.0).into(),
                    rotation: glam::Quat::IDENTITY,
                    scale: (1.0, 1.0, 1.0).into(),
                    mesh_id: MeshHandle(0)
                },
                ObjectState {
                    position: (2.0, 0.0, 0.0).into(),
                    rotation: glam::Quat::IDENTITY,
                    scale: (1.0, 1.0, 1.0).into(),
                    mesh_id: MeshHandle(0)
                },
                ObjectState {
                    position: (-2.0, 0.0, 0.0).into(),
                    rotation: glam::Quat::IDENTITY,
                    scale: (1.0, 2.0, 1.0).into(),
                    mesh_id: MeshHandle(0)
                }
            ]
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        for obj in &mut self.objects {
            obj.rotation *= glam::Quat::from_rotation_y(0.5 * dt);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.aspect = width as f32 / height as f32;
    }
}

#[derive(Debug)]
pub struct ObjectState {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
    pub mesh_id: MeshHandle,
}

impl ObjectState {
    pub fn to_model_matrix(&self) -> glam::Mat4 {
        let scale_mat = glam::Mat4::from_scale(self.scale);
        let rotation_mat = glam::Mat4::from_quat(self.rotation);
        let translation_mat = glam::Mat4::from_translation(self.position);

        translation_mat * rotation_mat * scale_mat
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
}