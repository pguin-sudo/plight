use anyhow::Result;
use confique::Config;
use serde::{Deserialize, Serialize};
use xcap::Monitor;

use crate::config::CONFIG;
use crate::core::led_color::LedColor;
use crate::modes::sources::Source;
use crate::utils::{converters::rgba8_to_rgb8, image_processing::parse_image};

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct ScreenSrcConf {
    #[config(default = "XCap")]
    pub engine: CaptureEngine,
}

pub struct ScreenSrc {
    monitor: Monitor,
}

impl ScreenSrc {
    pub fn new() -> Result<Self> {
        Ok(ScreenSrc {
            monitor: Monitor::all()?[0].clone(),
        })
    }
}

impl Source for ScreenSrc {
    fn poll_next(&mut self) -> Result<Vec<LedColor>> {
        let image = match CONFIG.source.screen.engine {
            CaptureEngine::XCap => self.monitor.capture_image()?,
        };

        Ok(parse_image(&rgba8_to_rgb8(image)))
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum CaptureEngine {
    XCap,
}
