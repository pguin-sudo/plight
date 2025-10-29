use std::ops::Mul;

use image::Pixel;
use image::Rgb;

use crate::config::CONFIG;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct LedColor(Rgb<u8>);

impl LedColor {
    pub fn apply_tint(&self) -> [u8; 3] {
        let rgb = self.0 .0;

        let r = self.apply_gamma(rgb[0], CONFIG.strip.tint.gamma[0]);
        let g = self.apply_gamma(rgb[1], CONFIG.strip.tint.gamma[1]);
        let b = self.apply_gamma(rgb[2], CONFIG.strip.tint.gamma[2]);

        let (r, g, b) = self.adjust_saturation(r, g, b, &CONFIG.strip.tint.saturation);

        let r = self.apply_brightness(r, CONFIG.strip.tint.brightness[0]);
        let g = self.apply_brightness(g, CONFIG.strip.tint.brightness[1]);
        let b = self.apply_brightness(b, CONFIG.strip.tint.brightness[2]);

        let (r, g, b) = self.apply_order(r, g, b);

        [r, g, b]
    }

    fn apply_gamma(&self, value: u8, gamma: f32) -> u8 {
        let normalized = value as f32 / 255.0;
        let corrected = normalized.powf(1.0 / gamma);
        (corrected * 255.0).round() as u8
    }

    fn adjust_saturation(&self, r: u8, g: u8, b: u8, saturation: &[f32; 3]) -> (u8, u8, u8) {
        let avg = (r as f32 + g as f32 + b as f32) / 3.0;
        let new_r = avg + saturation[0] * (r as f32 - avg);
        let new_g = avg + saturation[1] * (g as f32 - avg);
        let new_b = avg + saturation[2] * (b as f32 - avg);
        (
            new_r.clamp(0.0, 255.0) as u8,
            new_g.clamp(0.0, 255.0) as u8,
            new_b.clamp(0.0, 255.0) as u8,
        )
    }

    fn apply_brightness(&self, value: u8, brightness: f32) -> u8 {
        (brightness * value as f32).clamp(0.0, 255.0).round() as u8
    }

    fn apply_order(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        match CONFIG.strip.tint.order.as_str() {
            "RGB" => (r, g, b),
            "GRB" => (g, r, b),
            "BRG" => (b, r, g),
            "BGR" => (b, g, r),
            "RBG" => (r, b, g),
            _ => (r, g, b),
        }
    }
}

impl From<Rgb<u8>> for LedColor {
    fn from(color: Rgb<u8>) -> Self {
        LedColor(color)
    }
}

impl From<[u8; 3]> for LedColor {
    fn from(color: [u8; 3]) -> Self {
        LedColor(Rgb::from(color))
    }
}

impl Default for LedColor {
    fn default() -> Self {
        LedColor(Rgb::from([0_u8, 0_u8, 0_u8]))
    }
}

impl Mul<f64> for LedColor {
    type Output = LedColor;

    fn mul(self, rhs: f64) -> Self {
        LedColor(self.0.map(|x| (x as f64 * rhs) as u8))
    }
}
