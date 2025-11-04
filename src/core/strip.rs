use crate::core::led_sequence::LedSequence;
use anyhow::Result;

pub trait Strip {
    fn new() -> Result<Self>
    where
        Self: Sized;

    fn set_leds(&self, led_colors: &LedSequence) -> Result<()>;
}
