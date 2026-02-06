pub struct Player {
    pub position: glam::Vec3,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::new(2.0, 3.0, 0.0)
        }
    }
}