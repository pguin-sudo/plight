use std::sync::Mutex;

use confique::Config;
use image::Rgb;
use tokio::time::{sleep, Duration};

use crate::{config::CONFIG, modes::Mode, strip::Strip};

#[derive(Config)]
pub struct ColorModConf {
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,

    #[config(default = [192, 168, 31])]
    pub color: [u8; 3],
}

impl Mode {
    pub async fn poll_color(&self, strip: Mutex<Strip>) {
        let length: usize = CONFIG.strip.len().into();
        loop {
            let [r, g, b] = CONFIG.modes.color.color;
            let colors = vec![Rgb::<u8>::from([r, g, b]); length];
            strip.lock().unwrap().set_leds(&colors);
            sleep(Duration::from_millis(CONFIG.modes.color.update_rate)).await;
        }
    }
}
