use std::ops::Range;

use glam::{Vec3, Vec4Swizzles};

use crate::euc::{
    self,
    math::{Unit, WeightedSum},
    pipeline::{AaMode, DepthMode, Pipeline, PixelMode},
    primitives::{LineTriangleList, TriangleList},
    rasterizer::CullMode,
};

#[derive(Debug)]
pub struct VertexIn {
    pub pos: Vec3,
    pub normal: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct SurfaceVertexOut {
    pub clip_pos: Vec3,
    pub clip_normal: Vec3,
    pub vertex_color: Rgba,
    pub light_pos: Vec3,
}

impl WeightedSum for SurfaceVertexOut {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let mut clip_pos = Vec3::ZERO;
        let mut clip_normal = Vec3::ZERO;
        let mut light_pos = Vec3::ZERO;
        let mut r: f32 = 0.0;
        let mut g: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut a: f32 = 0.0;

        for i in 0..N {
            let w = weights[i];
            clip_pos += values[i].clip_pos * w;
            clip_normal += values[i].clip_normal * w;
            light_pos += values[i].light_pos * w;
            r += values[i].vertex_color.0[0] as f32 * w;
            g += values[i].vertex_color.0[1] as f32 * w;
            b += values[i].vertex_color.0[2] as f32 * w;
            a += values[i].vertex_color.0[3] as f32 * w;
        }

        Self {
            clip_pos,
            clip_normal,
            light_pos,
            vertex_color: Rgba([r as u8, g as u8, b as u8, a as u8]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WireframeVertexOut {
    pub clip_pos: Vec3,
    pub clip_normal: Vec3,
    pub vertex_color: Rgba,
}

impl WeightedSum for WireframeVertexOut {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let mut clip_pos = Vec3::ZERO;
        let mut clip_normal = Vec3::ZERO;
        let mut r: f32 = 0.0;
        let mut g: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut a: f32 = 0.0;

        for i in 0..N {
            let w = weights[i];
            clip_pos += values[i].clip_pos * w;
            clip_normal += values[i].clip_normal * w;
            r += values[i].vertex_color.0[0] as f32 * w;
            g += values[i].vertex_color.0[1] as f32 * w;
            b += values[i].vertex_color.0[2] as f32 * w;
            a += values[i].vertex_color.0[3] as f32 * w;
        }

        Self {
            clip_pos,
            clip_normal,
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
    pub model_matrix: glam::Mat4,
    pub light_matrix: glam::Mat4,
    pub camera_pos: Vec3,
    pub light_pos: Vec3,
}

impl<'r> Pipeline<'r> for ColoredMesh {
    type Vertex = VertexIn;

    type VertexData = SurfaceVertexOut;

    type Primitives = TriangleList;

    type Fragment = Rgba;

    type Pixel = Rgba;

    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let pos = glam::Vec4::new(vertex.pos[0], vertex.pos[1], vertex.pos[2], 1.0);
        let normal = glam::Vec4::new(vertex.normal[0], vertex.normal[1], vertex.normal[2], 1.0);
        let clip_pos = self.model_matrix * pos;
        let clip_normal = self.model_matrix * normal;

        let light_view_mat = self.light_matrix * pos;
        let light_view_pos = light_view_mat.xyz() / light_view_mat.w;
        (
            [clip_pos.x, clip_pos.y, clip_pos.z, clip_pos.w],
            SurfaceVertexOut {
                clip_pos: glam::Vec3::new(clip_pos.x, clip_pos.y, clip_pos.z),
                clip_normal: glam::Vec3::new(clip_normal.x, clip_normal.y, clip_normal.z),
                light_pos: light_view_pos,
                vertex_color: self.mesh_color,
            },
        )
    }

    fn fragment(&self, vs_out: Self::VertexData) -> Self::Fragment {
        //vs_out.vertex_color

        let wnorm = vs_out.clip_normal.normalize();
        let cam_dir = (self.camera_pos - vs_out.clip_pos).normalize();
        let light_dir = (vs_out.clip_pos - self.light_pos).normalize();
        let surf_color = Rgba([255, 156, 160, 255]);

        // Phong reflection model
        let ambient = 0.1;
        let diffuse = wnorm.dot(-light_dir).max(0.0) * 0.5;
        let specular = (-light_dir)
            .reflect(wnorm)
            .dot(-cam_dir)
            .max(0.0)
            .powf(30.0)
            * 3.0;

        // Shadow-mapping
        // let light_depth = self
        //     .shadow
        //     .sample((light_view_pos.xy() * Vec2::new(1.0, -1.0) * 0.5 + 0.5).into_array())
        //     + 0.0001;
        // let depth = light_view_pos.z;
        //let in_light = depth < light_depth;
        let in_light = false;

        let light = ambient
            + if in_light {
                diffuse + specular + 1.0
            } else {
                0.0
            };

        let r = (vs_out.vertex_color.0[0] as f32 * light).clamp(0.0, 255.0);
        let g = (vs_out.vertex_color.0[1] as f32 * light).clamp(0.0, 255.0);
        let b = (vs_out.vertex_color.0[2] as f32 * light).clamp(0.0, 255.0);
        Rgba([r as u8, g as u8, b as u8, 255])
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
    type Vertex = VertexIn;

    type VertexData = WireframeVertexOut;

    type Primitives = LineTriangleList;

    type Fragment = Rgba;

    type Pixel = Rgba;

    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let pos_vec = glam::Vec4::new(vertex.pos[0], vertex.pos[1], vertex.pos[2], 1.0);
        let transformed = self.mvp_matrix * pos_vec;
        (
            [transformed.x, transformed.y, transformed.z, transformed.w],
            WireframeVertexOut {
                clip_pos: glam::Vec3::new(transformed.x, transformed.y, transformed.z),
                clip_normal: glam::Vec3::new(transformed.x, transformed.y, transformed.z),
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

pub struct MeshShadow {
    camera_matrix: glam::Mat4,
}

impl<'r> Pipeline<'r> for MeshShadow {
    type Vertex = VertexIn;
    type VertexData = f32;
    type Primitives = TriangleList;
    type Fragment = Unit;
    type Pixel = ();

    #[inline(always)]
    fn pixel_mode(&self) -> PixelMode {
        PixelMode::PASS
    }

    #[inline(always)]
    fn depth_mode(&self) -> DepthMode {
        DepthMode::LESS_WRITE
    }

    #[inline(always)]
    fn rasterizer_config(&self) -> CullMode {
        CullMode::None
    }

    #[inline(always)]
    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let shadow_matrix =
            self.camera_matrix * glam::Vec4::new(vertex.pos.x, vertex.pos.y, vertex.pos.z, 1.0);
        (shadow_matrix.to_array(), 0.0)
    }

    #[inline(always)]
    fn fragment(&self, _: Self::VertexData) -> Self::Fragment {
        Unit
    }

    #[inline(always)]
    fn blend(&self, _old: Self::Pixel, _new: Self::Fragment) {}
}
