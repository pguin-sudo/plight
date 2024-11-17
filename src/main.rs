mod config;
mod errors;
mod modes;
mod strip;
mod utils;

use config::CONFIG;
use errors::Error::{
    Config, ImageError, ParseIntError, SerialPort, Utf8Error, VarError, XCapError,
};
use errors::Error::{PostfixReading, WrongLength, WrongPostfix};
use errors::Result;
use strip::Strip;

#[tokio::main]
async fn main() -> Result<()> {
    let mut strip = Strip::new(&CONFIG.strip)?;
    println!("Strip has set up successfully");

    let mode = CONFIG.mode;
    println!("Current mode is \"{:?}\"", mode);

    loop {
        // Start polling
        if let Err(e) = mode.poll(&mut strip).await {
            match e {
                WrongLength {
                    given: _,
                    actual: _,
                } => panic!("{}", e),
                PostfixReading(_) => println!("{}", e),
                ParseIntError(_) => println!("{}", e),
                WrongPostfix(_) => println!("{}", e),
                ImageError(_) => println!("{}", e),
                SerialPort(_) => panic!("{}", e),
                Utf8Error(_) => panic!("{}", e),
                XCapError(_) => panic!("{}", e),
                VarError(_) => panic!("{}", e),
                Config(_) => panic!("{}", e),
            }
        }
    }
}
