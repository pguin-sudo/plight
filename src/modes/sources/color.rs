use anyhow::Result;
use confique::Config;

use crate::config::CONFIG;
use crate::core::led_color::LedColor;
use crate::core::led_sequence::LedSequence;
use crate::modes::sources::Source;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct ColorSrcConf {
    #[config(default = [192, 168, 31])]
    pub color: [u8; 3],
}

pub struct ColorSrc {}

impl ColorSrc {
    pub fn new() -> Result<Self> {
        Ok(ColorSrc {})
    }
}

impl Source for ColorSrc {
    fn poll_next(&mut self, led_sequence: &mut LedSequence) -> Result<()> {
        let color = LedColor::from(CONFIG.source.color.color);

        led_sequence.set_color(color);
        Ok(())
    }
}
