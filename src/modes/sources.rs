pub mod color;
pub mod screen;
pub mod wallpaper;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    core::led_color::LedColor,
    modes::sources::{color::ColorSrc, screen::ScreenSrc, wallpaper::WallpaperSrc},
};

pub trait Source {
    fn poll_next(&mut self) -> Result<Vec<LedColor>>;
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum SourceMod {
    Color,
    Screen,
    Wallpaper,
}

impl SourceMod {
    pub async fn get_source(&self) -> Result<Box<dyn Source>> {
        match self {
            SourceMod::Color => Ok(Box::new(ColorSrc::new()?)),
            SourceMod::Screen => Ok(Box::new(ScreenSrc::new()?)),
            SourceMod::Wallpaper => Ok(Box::new(WallpaperSrc::new()?)),
        }
    }
}
