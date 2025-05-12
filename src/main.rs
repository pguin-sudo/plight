mod config;
mod errors;
mod modes;
mod strip;
mod utils;

use std::sync::Arc;

use config::CONFIG;
use errors::Result;
use strip::Strip;

#[tokio::main]
async fn main() -> Result<()> {
    let mut strip = Strip::new(&CONFIG.strip)?;
    let strip_arc = Arc::new(strip.clone());
    println!("Strip has set up successfully");

    let mode = CONFIG.mode;
    println!("Current mode is \"{:?}\"", mode);

    loop {
        if let Err(e) = mode.poll(strip_arc.clone(), &mut strip).await {
            println!("New loop after error: {:?}", e);
        }
    }
}
