use anyhow::Result;
use colog::init;
use log::info;
use plight::core::strip::Strip;

use plight::config::CONFIG;
use plight::core::arduino_strip::ArduinoStrip;
use plight::core::poll;

#[tokio::main]
async fn main() -> Result<()> {
    init();

    let source_mode = CONFIG.source;
    info!("Current color mode is \"{:?}\"", source_mode);
    let behavior_mode = CONFIG.behavior;
    info!("Current behavior mode is \"{:?}\"", behavior_mode);

    let strip = Box::new(ArduinoStrip::new()?);

    let _ = poll(strip, source_mode, behavior_mode).await;
    Ok(())
}
