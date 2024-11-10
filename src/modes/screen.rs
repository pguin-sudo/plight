use confique::Config;
use std::str;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};
use xcap::Monitor;

use crate::config::CONFIG;
use crate::strip::Strip;
use crate::utils::{average_color, parse_image, rgba8_to_rgb8};
use crate::modes::Mode;

#[derive(Config)]
pub struct ScreenModConf {
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,
}

impl Mode {
    pub async fn poll_screen(&self, strip: Mutex<Strip>) {
        let monitor = Monitor::all().unwrap()[0].clone();

        loop {
            let image = monitor.capture_image().unwrap();
            // ? Maybe there is better way to convert buffer to buffer without alpha
            strip.lock().unwrap().set_leds(&parse_image(&rgba8_to_rgb8(image), average_color).await);

            sleep(Duration::from_millis(CONFIG.modes.screen.update_rate)).await;
        }
    }
}

