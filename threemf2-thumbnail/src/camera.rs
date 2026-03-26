use glam::{Mat4, Vec3};

use crate::bbox::BoundingBox;

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
            up: Vec3::NEG_Z, //to make the camera oriented with Z axis upwards
        }
    }

    pub fn with_angles(mut self, yaw_deg: f32, pitch_deg: f32) -> Self {
        self.yaw = yaw_deg.to_radians();
        self.pitch = pitch_deg.clamp(-89.99, 89.99).to_radians();
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
        let depth = bounds_size.length() * padding_factor + 1.0;
        self.far = depth;
        self.near = -depth;
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

    pub fn position(&self) -> Vec3 {
        let direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
        .normalize();
        self.target - direction
    }

    pub fn get_directional_light_data(
        &self,
        scene_bounds: &BoundingBox,
        back_offset: f32,
        right_offset: f32,
    ) -> LightData {
        let camera_position = self.position();

        // Get camera direction and calculate orthogonal vectors
        let forward = (self.target - camera_position).normalize(); // Where camera looks
        let up = glam::Vec3::NEG_Z; // Same as camera
        let right = forward.cross(up).normalize(); // Camera's right vector

        // Light from back of camera (opposite to forward), slightly to the right
        let back_offset = -forward * back_offset; // Behind camera (negative forward)
        let right_offset = right * right_offset; // Slightly to the right
        let light_pos = camera_position + back_offset + right_offset;

        let light_dir = (self.target - light_pos).normalize(); // Direction from light to center

        let light_view = glam::Mat4::look_at_rh(light_pos, self.target, glam::Vec3::NEG_Z);

        // Use orthographic projection for the shadow map
        // 2. Transform the scene bounding box corners to light space
        let mut min = glam::Vec3::MAX;
        let mut max = glam::Vec3::MIN;
        for corner in scene_bounds.corners() {
            let light_space = light_view.transform_point3(corner);
            min = min.min(light_space);
            max = max.max(light_space);
        }

        // 3. Create orthographic projection covering the bounds
        let light_proj = glam::Mat4::orthographic_rh(
            min.x,
            max.x,
            min.y,
            max.y,
            min.z - 10.0, // near (add margin behind the light)
            max.z + 10.0, // far (add margin beyond the scene)
        );
        let light_view_proj = light_proj * light_view;

        LightData {
            light_pos,
            light_dir,
            light_view_proj,
        }
    }
}

pub struct LightData {
    pub light_pos: glam::Vec3,
    pub light_dir: glam::Vec3,
    pub light_view_proj: glam::Mat4,
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
