pub mod arduino_strip;
pub mod led_color;
pub mod led_sequence;
pub mod strip;

use anyhow::Result;

use crate::config::CONFIG;
use crate::core::led_sequence::LedSequence;
use crate::modes::behaviors::BehaviorMod;
use crate::modes::sources::SourceMod;
use strip::Strip;

pub fn poll(strip: Box<dyn Strip>, source_mod: SourceMod, behavior_mod: BehaviorMod) -> Result<()> {
    let mut source = source_mod.get_source()?;
    let mut behavior = behavior_mod.get_behavior(strip)?;

    let mut led_sequence = LedSequence::new(CONFIG.strip.len());

    loop {
        source.poll_next(&mut led_sequence)?;
        behavior.poll_next(&led_sequence)?;
    }
}
