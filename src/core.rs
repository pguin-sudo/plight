pub mod arduino_strip;
pub mod led_color;
pub mod strip;

use anyhow::Result;
use log::trace;

use crate::modes::behaviors::BehaviorMod;
use crate::modes::sources::SourceMod;
use strip::Strip;

pub async fn poll(
    strip: Box<dyn Strip>,
    source_mod: SourceMod,
    behavior_mod: BehaviorMod,
) -> Result<()> {
    let mut source = source_mod.get_source().await?;
    let mut behavior = behavior_mod.get_behavior(strip)?;

    loop {
        let colors = source.poll_next()?;
        let _ = behavior.poll_next(&colors);
        trace!("Color updated! New color is {:?}", colors[0]);
    }
}
