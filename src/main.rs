mod config;
mod modes;
mod strip;
mod utils;

use std::time::Duration;
use tokio::time::sleep;

use config::Conf;
use strip::Strip;

#[tokio::main]
async fn main() {
    let config = Conf::new();
    println!("Config has loaded successfully");

    let mut strip = Strip::new(&config.strip);
    println!("Strip has set up successfully");

    let mode = config.mode;
    println!("Current mode is \"{:?}\"", mode);

    loop {
        strip.set_leds(mode.process(&config));
        sleep(Duration::from_millis(config.update_rate)).await;
    }
}
