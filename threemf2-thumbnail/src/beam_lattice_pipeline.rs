use glam::{Vec3, Vec4Swizzles};

use crate::{
    euc::{
        math::WeightedSum,
        pipeline::{self, DepthMode, Pipeline},
        primitives::LineList,
    },
    rgba::Rgba,
};

pub struct BeamVertexIn {
    pub pos: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct BeamVertexOut {
    pub world_pos: Vec3,
    pub color: Rgba,
}

impl WeightedSum for BeamVertexOut {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let mut world_pos = Vec3::ZERO;
        let mut r: f32 = 0.0;
        let mut g: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut a: f32 = 0.0;

        for i in 0..N {
            let w = weights[i];
            world_pos += values[i].world_pos * w;
            r += values[i].color.0[0] as f32 * w;
            g += values[i].color.0[1] as f32 * w;
            b += values[i].color.0[2] as f32 * w;
            a += values[i].color.0[3] as f32 * w;
        }

        Self {
            world_pos,
            color: Rgba([r as u8, g as u8, b as u8, a as u8]),
        }
    }
}

pub struct ColoredBeamLattice {
    pub transform: glam::Mat4,
    pub view_proj: glam::Mat4,
    pub color: Rgba,
}

impl<'r> Pipeline<'r> for ColoredBeamLattice {
    type Vertex = BeamVertexIn;

    type VertexData = BeamVertexOut;

    type Primitives = LineList;

    type Fragment = Rgba;

    type Pixel = Rgba;

    fn depth_mode(&self) -> DepthMode {
        DepthMode::LESS_WRITE
    }

    fn vertex(&self, vs_in: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let local_pos = glam::Vec4::new(vs_in.pos.x, vs_in.pos.y, vs_in.pos.y, 1.0);
        let world_pos = self.transform * local_pos;
        let clip_pos = self.view_proj * world_pos;

        (
            clip_pos.to_array(),
            BeamVertexOut {
                world_pos: world_pos.xyz(),
                color: self.color,
            },
        )
    }

    fn fragment(&self, vs_out: Self::VertexData) -> Self::Fragment {
        vs_out.color
    }

    fn blend(&self, _old: Self::Pixel, new: Self::Fragment) -> Self::Pixel {
        new
    }
}
