use std::sync::{Arc, Mutex};
use std::time::Duration;

use image::Rgb;
use rand::random;
use serialport::{self, SerialPort};

use crate::config::{StripConf, TintConf};
use crate::errors::Error::{PostfixReading, WrongLength, WrongPostfix};
use crate::errors::Result;

const PREFIX: [u8; 3] = [89, 124, 234];

#[derive(Clone)]
pub struct Strip {
    port: Arc<Mutex<Box<dyn SerialPort + Send>>>,
    tint_conf: TintConf,
    strip_length: usize,
}

impl Strip {
    pub fn new(conf: &StripConf) -> Result<Strip> {
        let port: Box<dyn SerialPort + Send> =
            serialport::new(conf.serial_port.clone(), conf.baudrate)
                .timeout(Duration::from_millis(1000))
                .open()?;

        Ok(Strip {
            port: Arc::new(Mutex::new(port)),
            tint_conf: conf.tint.clone(),
            strip_length: conf.len(),
        })
    }

    pub fn set_leds(&self, led_colors: &[Rgb<u8>]) -> Result<()> {
        if led_colors.len() != self.strip_length {
            return Err(WrongLength {
                given: led_colors.len(),
                actual: self.strip_length,
            });
        }

        // TODO: Try to replace unwrap() with ?
        let mut port = self.port.lock().unwrap();

        port.write_all(&PREFIX)?;

        let hi: u8 = random();
        let lo: u8 = random();
        let chk = (hi ^ lo ^ 0x55) as u8;

        port.write_all(&[hi])?;
        port.write_all(&[lo])?;
        port.write_all(&[chk])?;

        for rgb in led_colors {
            let (r, g, b) = match self.tint_conf.order.as_str() {
                "RGB" => (rgb.0[0], rgb.0[1], rgb.0[2]),
                "GRB" => (rgb.0[1], rgb.0[0], rgb.0[2]),
                "BRG" => (rgb.0[2], rgb.0[0], rgb.0[1]),
                "BGR" => (rgb.0[2], rgb.0[1], rgb.0[0]),
                "RBG" => (rgb.0[0], rgb.0[2], rgb.0[1]),
                _ => (rgb.0[0], rgb.0[1], rgb.0[2]),
            };

            let (r, g, b) = self.apply_tint(r, g, b);
            port.write_all(&[r, g, b])?;
        }

        let mut buf = [0; 3];
        match port.read(&mut buf) {
            Ok(_) => {
                buf.reverse();
                if buf == PREFIX {
                    Ok(())
                } else {
                    Err(WrongPostfix(buf))
                }
            }
            Err(e) => Err(PostfixReading(e)),
        }
    }

    fn apply_tint(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let r = self.apply_gamma(r, self.tint_conf.gamma[0]);
        let g = self.apply_gamma(g, self.tint_conf.gamma[1]);
        let b = self.apply_gamma(b, self.tint_conf.gamma[2]);

        let (r, g, b) = self.adjust_saturation(r, g, b, self.tint_conf.saturation);

        let r = self.apply_brightness(r, self.tint_conf.brightness[0]);
        let g = self.apply_brightness(g, self.tint_conf.brightness[1]);
        let b = self.apply_brightness(b, self.tint_conf.brightness[2]);

        (r, g, b)
    }

    fn apply_gamma(&self, value: u8, gamma: f32) -> u8 {
        let normalized = value as f32 / 255.0;
        let corrected = normalized.powf(1.0 / gamma);
        (corrected * 255.0).round() as u8
    }

    fn adjust_saturation(&self, r: u8, g: u8, b: u8, saturation: [f32; 3]) -> (u8, u8, u8) {
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
        (brightness * value as f32).round() as u8
    }
}
