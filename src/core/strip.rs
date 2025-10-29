use crate::core::led_color::LedColor;
use anyhow::Result;

pub trait Strip {
    fn new() -> Result<Self>
    where
        Self: Sized;

    fn set_leds(&self, led_colors: &[LedColor]) -> Result<()>;
}
