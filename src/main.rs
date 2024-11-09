mod config;
mod modes;
mod strip;
mod utils;

use config::CONFIG;
use strip::Strip;

#[tokio::main]
async fn main() {
    let mut strip = Strip::new(&CONFIG.read().await.strip);
    println!("Strip has set up successfully");

    let mode = CONFIG.read().await.mode;
    println!("Current mode is \"{:?}\"", mode);

    // Start polling
    mode.poll(|f| strip.set_leds(f)).await;
}
