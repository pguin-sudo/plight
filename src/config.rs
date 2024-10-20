use confique::{toml::FormatOptions, Config};
use std::fs::File;
use std::io::Write;

use crate::modes::Mode;

const DEFAULT_CONFIG_PATH: &str = "/etc/plight/config.toml";

#[derive(Config)]
pub struct Conf {
    #[config(default = "Color")]
    pub mode: Mode,
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,

    // Strip configuration
    #[config(nested)]
    pub strip: StripConf,

    // Several PLight modes config
    #[config(nested)]
    pub modes: ModesConf,
}

#[derive(Clone, Config)]
pub struct StripConf {
    // #[config(default = true)]
    // pub skip_corners: bool,
    #[config(default = 29)]
    pub width: u8,
    #[config(default = 15)]
    pub hight: u8,
    #[config(default = 7)]
    pub bottom_gap: u8,
    // #[config(default = false)]
    // pub clockwise: bool,
    #[config(default = "/dev/ttyUSB0")]
    pub serial_port: String,
    #[config(default = 115200)]
    pub baudrate: u32,

    // Tint configuration
    #[config(nested)]
    pub tint: TintConf,
}

#[derive(Clone, Config)]
pub struct TintConf {
    #[config(default = "GRB")]
    pub order: String,
    #[config(default = 0.2)]
    pub gamma: f32,
    #[config(default = 1)]
    pub contrast: f32,
    #[config(default = 0.9)]
    pub saturation: f32,
}

#[derive(Config)]
pub struct ModesConf {
    #[config(nested)]
    pub color: ColorConf,
}

#[derive(Config)]
pub struct ColorConf {
    #[config(default = [192, 168, 31])]
    pub color: [u8; 3],
}

impl Conf {
    pub fn new() -> Conf {
        Conf::from_file(DEFAULT_CONFIG_PATH).expect("Error loading config")
    }
}

impl StripConf {
    pub fn length(&self) -> u8 {
        (self.width * 2 + self.hight * 2 - self.bottom_gap).into()
    }
}

// fn create_new_config() {
//     let _ = File::create(DEFAULT_CONFIG_PATH)
//         .expect(&format!("Creating new config in {}", DEFAULT_CONFIG_PATH))
//         .write_all(confique::toml::template::<Conf>(FormatOptions::default()).as_bytes());
//     println!("A new config has been created at `{}`", DEFAULT_CONFIG_PATH);
// }
