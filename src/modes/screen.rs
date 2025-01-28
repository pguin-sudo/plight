use confique::Config;
use serde::{Deserialize, Serialize};
use std::str;
use xcap::Monitor;

use crate::config::CONFIG;
use crate::errors::Result;
use crate::modes::Mode;
use crate::strip::Strip;
use crate::utils::{parse_image, rgba8_to_rgb8};

#[derive(Config)]
pub struct ScreenModConf {
    #[config(default = "XCap")]
    pub engine: CaptureEngine,
}

impl Mode {
    pub async fn poll_screen(&self, strip: &mut Strip) -> Result<()> {
        let monitor = Monitor::all()?[0].clone();

        loop {
            let image = match CONFIG.modes.screen.engine {
                CaptureEngine::XCap => monitor.capture_image()?,
            };

            // ? Maybe there is better way to convert buffer to buffer without alpha
            strip.set_leds(&parse_image(&rgba8_to_rgb8(image)).await)?;
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum CaptureEngine {
    XCap,
}
