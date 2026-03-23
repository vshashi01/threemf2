use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct OrthographicCamera {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    aspect_ratio: f32,
    view_width: f32,
    view_height: f32,
    near: f32,
    far: f32,
    up: Vec3,
}

impl OrthographicCamera {
    pub fn looking_at(target: Vec3) -> Self {
        Self {
            target,
            yaw: 0.0,
            pitch: 0.0,
            aspect_ratio: 1.0,
            view_width: 2.0,
            view_height: 2.0,
            near: -1000.0,
            far: 1000.0,
            up: Vec3::NEG_Z, //to make the camera go up
        }
    }

    pub fn with_angles(mut self, yaw_deg: f32, pitch_deg: f32) -> Self {
        self.yaw = yaw_deg.to_radians();
        self.pitch = pitch_deg.clamp(-89.0, 89.0).to_radians();
        self
    }

    pub fn with_aspect_ratio(mut self, aspect: f32) -> Self {
        self.aspect_ratio = aspect;
        self
    }

    pub fn fit_to_bounds(&mut self, bounds_size: Vec3, padding: f32) {
        let padding_factor = 1.0 + padding;
        let diagonal = bounds_size.length() * padding_factor;
        let max_dimension = bounds_size.x.max(bounds_size.y).max(bounds_size.z) * padding_factor;

        self.view_width = diagonal.max(max_dimension);
        self.view_height = self.view_width / self.aspect_ratio;
        self.far = bounds_size.length() * padding_factor + 1.0;
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        let pos = self.position();
        let view = Mat4::look_at_rh(pos, self.target, self.up);
        let proj = Mat4::orthographic_rh(
            -self.view_width * 0.5,
            self.view_width * 0.5,
            -self.view_height * 0.5,
            self.view_height * 0.5,
            self.near,
            self.far,
        );
        proj * view
    }

    fn position(&self) -> Vec3 {
        let direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
        .normalize();
        self.target - direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_basic() {
        let camera = OrthographicCamera::looking_at(Vec3::ZERO)
            .with_angles(45.0, 30.0)
            .with_aspect_ratio(1.0);
        let mvp = camera.view_projection_matrix();
        assert!(mvp.is_finite());
    }
}
