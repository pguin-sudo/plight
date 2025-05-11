mod config;
mod errors;
mod modes;
mod strip;
mod utils;

use config::CONFIG;
use errors::Result;
use strip::Strip;

#[tokio::main]
async fn main() -> Result<()> {
    let mut strip = Strip::new(&CONFIG.strip)?;
    println!("Strip has set up successfully");

    let mode = CONFIG.mode;
    println!("Current mode is \"{:?}\"", mode);

    loop {
        if let Err(e) = mode.poll(&mut strip).await {
            println!("{:?}", e);
        }
    }
}
