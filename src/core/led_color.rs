use std::ops::Mul;

use image::Pixel;
use image::Rgb;
use unit_interval::UnitInterval;

use crate::config::CONFIG;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LedColor(Rgb<f32>);

impl LedColor {
    pub fn apply_tint(self) -> [u8; 3] {
        let rgb = self.0 .0;

        let r = self._adjust_gamma(rgb[0], CONFIG.strip.tint.gamma[0]);
        let g = self._adjust_gamma(rgb[1], CONFIG.strip.tint.gamma[1]);
        let b = self._adjust_gamma(rgb[2], CONFIG.strip.tint.gamma[2]);

        let (r, g, b) = self._adjust_saturation(r, g, b, &CONFIG.strip.tint.saturation);

        let g = self._adjust_brightness(g, CONFIG.strip.tint.brightness[1]);
        let b = self._adjust_brightness(b, CONFIG.strip.tint.brightness[2]);

        let (r, g, b) = self._adjust_order(r, g, b);

        [r as u8, g as u8, b as u8]
    }

    fn _adjust_gamma(&self, value: f32, gamma: f32) -> f32 {
        let normalized = value / 255.0;
        let corrected = normalized.powf(1.0 / gamma);
        (corrected * 255.0).round()
    }

    fn _adjust_saturation(&self, r: f32, g: f32, b: f32, saturation: &[f32; 3]) -> (f32, f32, f32) {
        let avg = (r + g + b) / 3.0;
        let new_r = avg + saturation[0] * (r - avg);
        let new_g = avg + saturation[1] * (g - avg);
        let new_b = avg + saturation[2] * (b - avg);
        (
            new_r.clamp(0.0, 255.0),
            new_g.clamp(0.0, 255.0),
            new_b.clamp(0.0, 255.0),
        )
    }

    fn _adjust_brightness(&self, value: f32, brightness: f32) -> f32 {
        (brightness * value).clamp(0.0, 255.0).round()
    }

    fn _adjust_order(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
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
        LedColor(Rgb([
            color.0[0] as f32,
            color.0[1] as f32,
            color.0[2] as f32,
        ]))
    }
}

impl From<[u8; 3]> for LedColor {
    fn from(color: [u8; 3]) -> Self {
        LedColor(Rgb([color[0] as f32, color[1] as f32, color[2] as f32]))
    }
}

impl From<Rgb<f32>> for LedColor {
    fn from(color: Rgb<f32>) -> Self {
        LedColor(color)
    }
}

impl From<[f32; 3]> for LedColor {
    fn from(color: [f32; 3]) -> Self {
        LedColor(Rgb::from(color))
    }
}

impl Default for LedColor {
    fn default() -> Self {
        LedColor(Rgb::from([0_f32, 0_f32, 0_f32]))
    }
}

impl Mul<UnitInterval<f64>> for LedColor {
    type Output = LedColor;

    fn mul(self, rhs: UnitInterval<f64>) -> Self {
        LedColor(self.0.map(|x| (x as f64 * rhs.as_inner()) as f32))
    }
}
