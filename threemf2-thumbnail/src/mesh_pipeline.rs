use std::ops::Range;

use glam::Vec3;

use crate::euc::{
    self,
    math::WeightedSum,
    pipeline::{AaMode, Pipeline},
    primitives::{LineTriangleList, TriangleList},
};

#[derive(Debug)]
pub struct InputVertexData {
    pub pos: [f32; 3],
    // pub normal: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct MeshVertexData {
    pub clip_pos: Vec3,
    pub world_pos: Vec3,
    pub vertex_color: Rgba,
}

impl WeightedSum for MeshVertexData {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let mut clip_pos = Vec3::ZERO;
        let mut world_pos = Vec3::ZERO;
        let mut r: f32 = 0.0;
        let mut g: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut a: f32 = 0.0;

        for i in 0..N {
            let w = weights[i];
            clip_pos += values[i].clip_pos * w;
            world_pos += values[i].world_pos * w;
            r += values[i].vertex_color.0[0] as f32 * w;
            g += values[i].vertex_color.0[1] as f32 * w;
            b += values[i].vertex_color.0[2] as f32 * w;
            a += values[i].vertex_color.0[3] as f32 * w;
        }

        Self {
            clip_pos,
            world_pos,
            vertex_color: Rgba([r as u8, g as u8, b as u8, a as u8]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgba(pub [u8; 4]);

impl WeightedSum for Rgba {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let mut r = 0.0f32;
        let mut g = 0.0f32;
        let mut b = 0.0f32;
        let mut a = 0.0f32;

        for i in 0..N {
            r += values[i].0[0] as f32 * weights[i];
            g += values[i].0[1] as f32 * weights[i];
            b += values[i].0[2] as f32 * weights[i];
            a += values[i].0[3] as f32 * weights[i];
        }

        Rgba([r as u8, g as u8, b as u8, a as u8])
    }
}

pub struct ColoredMesh {
    pub mesh_color: Rgba,
    // camera_pos: glam::Vec3,
    // view: glam::Mat4,
    // projection: glam::Mat4,
    pub mvp_matrix: glam::Mat4,
}

impl<'r> Pipeline<'r> for ColoredMesh {
    type Vertex = InputVertexData;

    type VertexData = MeshVertexData;

    type Primitives = TriangleList;

    type Fragment = Rgba;

    type Pixel = Rgba;

    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let pos_vec = glam::Vec4::new(vertex.pos[0], vertex.pos[1], vertex.pos[2], 1.0);
        let transformed = self.mvp_matrix * pos_vec;
        (
            [transformed.x, transformed.y, transformed.z, transformed.w],
            MeshVertexData {
                clip_pos: glam::Vec3::new(transformed.x, transformed.y, transformed.z),
                world_pos: glam::Vec3::new(transformed.x, transformed.y, transformed.z),
                vertex_color: self.mesh_color,
            },
        )
    }

    fn fragment(&self, vs_out: Self::VertexData) -> Self::Fragment {
        vs_out.vertex_color
    }

    fn blend(&self, _old: Self::Pixel, new: Self::Fragment) -> Self::Pixel {
        new
    }

    fn aa_mode(&self) -> crate::euc::pipeline::AaMode {
        AaMode::Msaa { level: 2 }
    }
}

pub struct WireframeMesh {
    pub wireframe_color: Rgba,
    // camera_pos: glam::Vec3,
    // view: glam::Mat4,
    // projection: glam::Mat4,
    pub mvp_matrix: glam::Mat4,
}

impl<'r> Pipeline<'r> for WireframeMesh {
    type Vertex = InputVertexData;

    type VertexData = MeshVertexData;

    type Primitives = LineTriangleList;

    type Fragment = Rgba;

    type Pixel = Rgba;

    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let pos_vec = glam::Vec4::new(vertex.pos[0], vertex.pos[1], vertex.pos[2], 1.0);
        let transformed = self.mvp_matrix * pos_vec;
        (
            [transformed.x, transformed.y, transformed.z, transformed.w],
            MeshVertexData {
                clip_pos: glam::Vec3::new(transformed.x, transformed.y, transformed.z),
                world_pos: glam::Vec3::new(transformed.x, transformed.y, transformed.z),
                vertex_color: self.wireframe_color,
            },
        )
    }

    fn fragment(&self, vs_out: Self::VertexData) -> Self::Fragment {
        vs_out.vertex_color
    }

    fn blend(&self, _old: Self::Pixel, new: Self::Fragment) -> Self::Pixel {
        new
    }

    fn aa_mode(&self) -> crate::euc::pipeline::AaMode {
        AaMode::Msaa { level: 2 }
    }
}
