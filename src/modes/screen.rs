use confique::Config;
use image::Rgb;
use std::str;
use tokio::time::{sleep, Duration};
use xcap::Monitor;

use crate::utils::{average_color, parse_image, rgba8_to_rgb8};
use crate::{config::Conf, modes::Mode};

#[derive(Config)]
pub struct ScreenModConf {
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,
}

impl Mode {
    pub async fn poll_screen<F>(&self, config: &Conf, mut draw: F)
    where
        F: FnMut(&[Rgb<u8>]),
    {
        let monitor = Monitor::all().unwrap()[0].clone();

        loop {
            let image = monitor.capture_image().unwrap();
            // ? Maybe there is better way to convert buffer to buffer without alpha
            draw(&parse_image(
                &rgba8_to_rgb8(image),
                average_color,
                &config.strip,
            ));

            sleep(Duration::from_millis(config.modes.screen.update_rate)).await;
        }
    }
}
