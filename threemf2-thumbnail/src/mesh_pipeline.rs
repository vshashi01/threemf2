use euc::Pipeline;
// use vek_old::{Mat4, Vec4};

/// euc Pipeline for rendering thumbnails
pub(crate) struct ColoredMeshPipeline {
    pub(crate) mvp_matrix: glam::Mat4,
    pub(crate) mesh_color: [u8; 4],
}

impl Pipeline for ColoredMeshPipeline {
    type Vertex = [f32; 3]; // x, y, z
    type VsOut = (); // No data passed from vertex to fragment
    type Pixel = [u8; 4]; // RGBA

    // Vertex shader: transform vertices to clip space
    fn vert(&self, pos: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let pos_vec = glam::Vec4::new(pos[0], pos[1], pos[2], 1.0);
        let transformed = self.mvp_matrix * pos_vec;
        (
            [transformed.x, transformed.y, transformed.z, transformed.w],
            (),
        )
    }

    // Fragment shader: output flat color
    fn frag(&self, _: &Self::VsOut) -> Self::Pixel {
        self.mesh_color
    }
}
