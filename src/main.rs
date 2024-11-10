mod config;
mod modes;
mod strip;
mod utils;

use std::sync::Mutex;

use config::CONFIG;
use strip::Strip;

#[tokio::main]
async fn main() {
    let strip = Mutex::new(Strip::new(&CONFIG.strip));
    println!("Strip has set up successfully");

    let mode = CONFIG.mode;
    println!("Current mode is \"{:?}\"", mode);

    // Start polling
    mode.poll(strip).await;
}
