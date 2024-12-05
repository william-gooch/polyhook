use bytemuck::{Pod, Zeroable};

pub struct Orbit {
    pub phi: f32,
    pub theta: f32,
    pub d: f32,
}

impl Orbit {
    pub fn update(&mut self, del: glam::Vec3) {
        self.theta -= del.x * 0.01;
        self.phi -= del.y * 0.01;
        self.d -= del.z * 0.005 * self.d;
    }

    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(glam::vec3(0.0, 0.0, self.d))
            * glam::Mat4::from_rotation_x(self.phi)
            * glam::Mat4::from_rotation_y(self.theta)
    }
}

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct Mvp {
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
    pub normal: glam::Mat4,
}

impl Mvp {
    const Z_NEAR: f32 = 0.01;

    pub fn new() -> Self {
        Self {
            model: glam::Mat4::IDENTITY,
            view: glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 3.0)),
            projection: glam::Mat4::perspective_infinite_lh(
                std::f32::consts::PI / 4.0,
                1.0,
                Mvp::Z_NEAR,
            ),
            normal: glam::Mat4::IDENTITY,
        }
    }

    #[allow(unused)]
    pub fn update_model(&mut self, model: glam::Mat4) {
        self.model = model;
        let (_, rot, _) = model.to_scale_rotation_translation();
        self.normal = glam::Mat4::from_quat(rot);
    }

    pub fn update_projection(&mut self, aspect_ratio: f32) {
        self.projection = glam::Mat4::perspective_infinite_lh(
            std::f32::consts::PI / 4.0,
            aspect_ratio,
            Mvp::Z_NEAR,
        );
    }
}
