pub struct Orbit {
    pub phi: f32,
    pub theta: f32,
    pub d: f32,
}

impl Orbit {
    pub fn update(&mut self, del: glam::Vec3) {
        self.theta -= del.x * 0.01;
        self.phi -= del.y * 0.01;
        self.d -= del.z * 0.01;
    }

    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(glam::vec3(0.0, 0.0, self.d))
            * glam::Mat4::from_rotation_x(self.phi)
            * glam::Mat4::from_rotation_y(self.theta)
    }
}

#[derive(Clone)]
pub struct MVP {
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}

impl MVP {
    const Z_NEAR: f32 = 0.01;
    const Z_FAR: f32 = 100.0;

    pub fn new() -> Self {
        Self {
            model: glam::Mat4::IDENTITY,
            view: glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 3.0)),
            projection: glam::Mat4::perspective_lh(
                std::f32::consts::PI / 4.0,
                1.0,
                MVP::Z_NEAR,
                MVP::Z_FAR,
            ),
        }
    }

    pub fn matrix(&self) -> glam::Mat4 {
        self.projection * self.view * self.model
    }

    pub fn update_projection(&mut self, aspect_ratio: f32) {
        self.projection = glam::Mat4::perspective_lh(
            std::f32::consts::PI / 4.0,
            aspect_ratio,
            MVP::Z_NEAR,
            MVP::Z_FAR,
        );
    }
}
