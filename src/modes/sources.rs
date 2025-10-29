// pub mod screen;
pub mod color;
// pub mod wallpaper;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{core::led_color::LedColor, modes::sources::color::ColorSrc};

pub trait Source {
    fn new() -> Result<impl Source>;

    async fn poll_next(&mut self) -> Result<Vec<LedColor>>;
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum SourceMod {
    Color,
}

impl SourceMod {
    pub async fn get_source(&self) -> Result<impl Source> {
        match self {
            SourceMod::Color => ColorSrc::new(),
        }
    }
}
