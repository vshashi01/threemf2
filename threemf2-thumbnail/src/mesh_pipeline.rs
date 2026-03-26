use glam::{Vec3, Vec4Swizzles};

use crate::euc::{
    self,
    math::WeightedSum,
    pipeline::{AaMode, DepthMode, Pipeline},
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
    pub world_pos: Vec3,
    pub world_normal: Vec3,
    pub color: Rgba,
    pub light_ndc: Vec3,
}

impl WeightedSum for SurfaceVertexOut {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let mut world_pos = Vec3::ZERO;
        let mut world_normal = Vec3::ZERO;
        let mut light_ndc = Vec3::ZERO;
        let mut r: f32 = 0.0;
        let mut g: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut a: f32 = 0.0;

        for i in 0..N {
            let w = weights[i];
            world_pos += values[i].world_pos * w;
            world_normal += values[i].world_normal * w;
            light_ndc += values[i].light_ndc * w;
            r += values[i].color.0[0] as f32 * w;
            g += values[i].color.0[1] as f32 * w;
            b += values[i].color.0[2] as f32 * w;
            a += values[i].color.0[3] as f32 * w;
        }

        Self {
            world_pos,
            world_normal,
            light_ndc,
            color: Rgba([r as u8, g as u8, b as u8, a as u8]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WireframeVertexOut {
    pub world_pos: Vec3,
    //pub world_normal: Vec3,
    pub vertex_color: Rgba,
}

impl WeightedSum for WireframeVertexOut {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let mut world_pos = Vec3::ZERO;
        //let mut world_normal = Vec3::ZERO;
        let mut r: f32 = 0.0;
        let mut g: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut a: f32 = 0.0;

        for i in 0..N {
            let w = weights[i];
            world_pos += values[i].world_pos * w;
            // world_normal += values[i].world_normal * w;
            r += values[i].vertex_color.0[0] as f32 * w;
            g += values[i].vertex_color.0[1] as f32 * w;
            b += values[i].vertex_color.0[2] as f32 * w;
            a += values[i].vertex_color.0[3] as f32 * w;
        }

        Self {
            world_pos,
            //world_normal,
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
    // Transforms
    pub model: glam::Mat4,           // Instance transforms
    pub view_proj: glam::Mat4,       // Projection * View
    pub light_view_proj: glam::Mat4, // LightProjection * LightView
    pub normal_matrix: glam::Mat3,   // transpose(inverse(model))

    // Camera
    pub camera_pos: Vec3,

    // Directional light
    pub light_dir: Vec3, // world space, normalized

    pub mesh_color: Rgba,
}

impl<'r> Pipeline<'r> for ColoredMesh {
    type Vertex = VertexIn;

    type VertexData = SurfaceVertexOut;

    type Primitives = TriangleList;

    type Fragment = Rgba;

    type Pixel = Rgba;

    fn depth_mode(&self) -> DepthMode {
        DepthMode::LESS_WRITE
    }

    fn vertex(&self, vs_in: &VertexIn) -> ([f32; 4], SurfaceVertexOut) {
        let local_pos = glam::Vec4::new(vs_in.pos[0], vs_in.pos[1], vs_in.pos[2], 1.0);

        // ---- Model → World
        let world_pos4 = self.model * local_pos;
        let world_pos = world_pos4.xyz();

        // ---- Normal → World
        let world_normal = (self.normal_matrix * vs_in.normal).normalize();

        // ---- World → Clip (camera)
        let clip_pos = self.view_proj * world_pos4;

        // ---- World → Light Clip (shadows)
        let light_clip = self.light_view_proj * world_pos4;
        let light_ndc = light_clip.xyz() / light_clip.w;

        (
            [clip_pos.x, clip_pos.y, clip_pos.z, clip_pos.w],
            SurfaceVertexOut {
                world_pos,
                world_normal,
                light_ndc,
                color: self.mesh_color,
            },
        )
    }

    fn fragment(&self, vs_out: Self::VertexData) -> Rgba {
        // ---- Lighting vectors (world space)
        let n = vs_out.world_normal.normalize();
        let l = (-self.light_dir).normalize();
        let v = (self.camera_pos - vs_out.world_pos).normalize();

        // ---- Phong (simple, thumbnail-friendly)
        let ambient = 0.15;
        let diffuse = n.dot(l).max(0.0);
        let specular = if diffuse > 0.0 {
            let r = (-l).reflect(n);
            r.dot(v).max(0.0).powf(32.0)
        } else {
            0.0
        };
        let light = ambient + diffuse + specular;

        // ---- Final color
        let c = vs_out.color.0;
        Rgba([
            (c[0] as f32 * light).clamp(0.0, 255.0) as u8,
            (c[1] as f32 * light).clamp(0.0, 255.0) as u8,
            (c[2] as f32 * light).clamp(0.0, 255.0) as u8,
            255,
        ])
    }

    fn blend(&self, _old: Self::Pixel, new: Self::Fragment) -> Self::Pixel {
        new
    }

    fn aa_mode(&self) -> crate::euc::pipeline::AaMode {
        AaMode::Msaa { level: 2 }
    }

    fn rasterizer_config(
            &self,
    ) -> <<Self::Primitives as euc::primitives::PrimitiveKind<Self::VertexData>>::Rasterizer as euc::rasterizer::Rasterizer>::Config{
        CullMode::None
    }
}

pub struct WireframeMesh {
    pub model: glam::Mat4,
    pub view_proj: glam::Mat4,
    pub wireframe_color: Rgba,
}

impl<'r> Pipeline<'r> for WireframeMesh {
    type Vertex = VertexIn;

    type VertexData = WireframeVertexOut;

    type Primitives = LineTriangleList;

    type Fragment = Rgba;

    type Pixel = Rgba;

    fn vertex(&self, vertex: &VertexIn) -> ([f32; 4], Self::VertexData) {
        let local_pos = glam::Vec4::new(vertex.pos.x, vertex.pos.y, vertex.pos.z, 1.0);
        let world_pos = self.model * local_pos;
        let clip_pos = self.view_proj * world_pos;

        //ToDo: consider introducing Wireframe bias to reduce the problem with Z fighting.
        (
            [clip_pos.x, clip_pos.y, clip_pos.z, clip_pos.w],
            WireframeVertexOut {
                world_pos: glam::Vec3::new(world_pos.x, world_pos.y, world_pos.z),
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
