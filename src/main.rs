use anyhow::Result;
use colog::init;
use log::{error, info};
use plight::core::strip::Strip;

use plight::config::CONFIG;
use plight::core::arduino_strip::ArduinoStrip;
use plight::core::poll;

fn main() -> Result<()> {
    init();

    let source_mode = CONFIG.source.mode;
    info!("Current source mode is \"{:?}\"", source_mode);
    let behavior_mode = CONFIG.behavior.mode;
    info!("Current behavior mode is \"{:?}\"", behavior_mode);

    let strip = Box::new(ArduinoStrip::new()?);

    if let Err(e) = poll(strip, source_mode, behavior_mode) {
        error!("PLight crushed with error: {:}", e);
    }
    Ok(())
}
