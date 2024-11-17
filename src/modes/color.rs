use confique::Config;
use image::Rgb;
use std::sync::Mutex;

use crate::config::CONFIG;
use crate::errors::Result;
use crate::modes::Mode;
use crate::strip::Strip;

#[derive(Config)]
pub struct ColorModConf {
    #[config(default = [192, 168, 31])]
    pub color: [u8; 3],
}

impl Mode {
    pub async fn poll_color(&self, strip: &Mutex<Strip>) -> Result<()> {
        let length: usize = CONFIG.strip.len().into();
        let mut prev_color = Rgb::from([0_u8, 0_u8, 0_u8]);
        loop {
            let color = Rgb::from(CONFIG.modes.color.color);

            if prev_color == color {
                continue;
            }

            prev_color = color;

            let colors = vec![color; length];
            strip.lock().unwrap().set_leds(&colors)?;
        }
    }
}
