use image::Rgb;
use rand::random;
use serialport::{self, SerialPort};

use crate::config::{StripConf, TintConf};

const PREFIX: [u8; 3] = [89, 124, 234];

pub struct Strip {
    port: Box<dyn SerialPort>,
    tint_conf: TintConf,
    strip_length: usize,
}

impl Strip {
    pub fn new(conf: &StripConf) -> Strip {
        Strip {
            port: serialport::new(conf.serial_port.clone(), conf.baudrate)
                .open()
                .expect("Failed to open port"),
            tint_conf: conf.tint.clone(),
            strip_length: conf.len() as usize,
        }
    }

    pub fn set_leds(&mut self, led_colors: &[Rgb<u8>]) {
        if led_colors.len() != self.strip_length {
            println!(
                "Wrong strip length to set ({} is not {})",
                led_colors.len(),
                self.strip_length
            );
            return;
        }

        let _ = self.port.write(&PREFIX);

        let hi: u8 = random();
        let lo: u8 = random();
        let chk = (hi ^ lo ^ 0x55) as u8;

        let _ = self.port.write(&[hi]);
        let _ = self.port.write(&[lo]);
        let _ = self.port.write(&[chk]);

        for rgb in led_colors {
            let (r, g, b) = match self.tint_conf.order.as_str() {
                "RGB" => (rgb.0[0], rgb.0[1], rgb.0[2]),
                "GRB" => (rgb.0[1], rgb.0[0], rgb.0[2]),
                "BRG" => (rgb.0[2], rgb.0[0], rgb.0[1]),
                "BGR" => (rgb.0[2], rgb.0[1], rgb.0[0]),
                "RBG" => (rgb.0[0], rgb.0[2], rgb.0[1]),
                _ => (rgb.0[0], rgb.0[1], rgb.0[2]),
            };

            let (r, g, b) = self.apply_gamma_contrast_saturation(r, g, b);
            let _ = self.port.write(&[r, g, b]);
        }
    }

    fn apply_gamma_contrast_saturation(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let r = self.apply_gamma(r, self.tint_conf.gamma);
        let g = self.apply_gamma(g, self.tint_conf.gamma);
        let b = self.apply_gamma(b, self.tint_conf.gamma);

        let (r, g, b) = self.adjust_contrast(r, g, b, self.tint_conf.contrast);
        let (r, g, b) = self.adjust_saturation(r, g, b, self.tint_conf.saturation);

        (r, g, b)
    }

    fn apply_gamma(&self, value: u8, gamma: f32) -> u8 {
        let normalized = value as f32 / 255.0;
        let corrected = normalized.powf(1.0 / gamma);
        (corrected * 255.0).round() as u8
    }

    fn adjust_contrast(&self, r: u8, g: u8, b: u8, contrast: f32) -> (u8, u8, u8) {
        let factor = (259.0 * (contrast + 255.0)) / (255.0 * (259.0 - contrast));
        let new_r = (factor * (r as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
        let new_g = (factor * (g as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
        let new_b = (factor * (b as f32 - 128.0) + 128.0).clamp(0.0, 255.0) as u8;
        (new_r, new_g, new_b)
    }

    fn adjust_saturation(&self, r: u8, g: u8, b: u8, saturation: f32) -> (u8, u8, u8) {
        let avg = (r as f32 + g as f32 + b as f32) / 3.0;
        let new_r = avg + saturation * (r as f32 - avg);
        let new_g = avg + saturation * (g as f32 - avg);
        let new_b = avg + saturation * (b as f32 - avg);
        (
            new_r.clamp(0.0, 255.0) as u8,
            new_g.clamp(0.0, 255.0) as u8,
            new_b.clamp(0.0, 255.0) as u8,
        )
    }
}
