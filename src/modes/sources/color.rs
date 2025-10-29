use anyhow::Result;
use confique::Config;

use crate::config::CONFIG;
use crate::core::led_color::LedColor;
use crate::modes::sources::Source;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct ColorSrcConf {
    #[config(default = [192, 168, 31])]
    pub color: [u8; 3],
}

pub struct ColorSrc {
    length: usize,
}

impl ColorSrc {
    pub fn new() -> Result<Self> {
        Ok(ColorSrc {
            length: CONFIG.strip.len(),
        })
    }
}

impl Source for ColorSrc {
    fn poll_next(&mut self) -> Result<Vec<LedColor>> {
        let color = LedColor::from(CONFIG.source.color.color);

        Ok(vec![color; self.length])
    }
}
