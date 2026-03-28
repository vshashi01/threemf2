use crate::euc::math::WeightedSum;

#[derive(Debug, Clone, Copy, PartialEq)]
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
