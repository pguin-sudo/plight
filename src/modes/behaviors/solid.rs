use anyhow::Result;
use confique::Config;

use crate::core::led_sequence::LedSequence;
use crate::core::strip::Strip;
use crate::modes::behaviors::Behavior;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct SolidBhvConf {}

pub struct SolidBhv {
    strip: Box<dyn Strip>,
}

impl SolidBhv {
    pub fn new(strip: Box<dyn Strip>) -> Result<Self> {
        Ok(SolidBhv { strip })
    }
}

impl Behavior for SolidBhv {
    fn poll_next(&mut self, colors: &LedSequence) -> Result<()> {
        self.strip.set_leds(colors)?;
        Ok(())
    }
}
