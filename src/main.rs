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

    loop {
        // Start polling
        match mode.poll(&strip).await {
            Ok(_) => todo!(),
            Err(e) => match e {
                strip::SetLedsError::WrongLength(a, b) => panic!("Wrong length {} {}", a, b),
                strip::SetLedsError::WrongPostfix(a) => println!("Wrong postfix error: {:?}", a),
                strip::SetLedsError::ReadPostfix(a) => println!("Read prefix error: {:?}", a),
            },
        }
    }
}
