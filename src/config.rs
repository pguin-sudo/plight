use confique::{toml::FormatOptions, Config};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;
use std::env;

use crate::modes::Mode;

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

impl StripConf {
    pub fn length(&self) -> u8 {
        (self.width * 2 + self.hight * 2 - self.bottom_gap).into()
    }
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
    pub color: ColorModConf,
    #[config(nested)]
    pub cava_wall_dcol: CavaWallDcolModConf,
}

#[derive(Config)]
pub struct ColorModConf {
    #[config(default = [192, 168, 31])]
    pub color: [u8; 3],
}

#[derive(Config)]
pub struct CavaWallDcolModConf {
    #[config(default = "/home/pguin/.config/cava/Wall-Dcol")]
    pub path_to_dcol: PathBuf,
}

impl Conf {
    pub fn new() -> Conf {
        let default_config_path = get_default_config_path();
        create_new_config(&default_config_path);
        Conf::from_file(default_config_path).expect("Error loading config")
    }
}

fn get_default_config_path() -> String {
    let home_dir = env::var("HOME").expect("Unable to get HOME directory");
    format!("{}/.config/plight/config.toml", home_dir)
}

fn create_new_config(default_config_path: &str) {
    let binding = PathBuf::from(default_config_path);
    let parent_dir = binding.parent().unwrap();
    if let Err(e) = fs::create_dir_all(parent_dir) {
        eprintln!("Error creating directories: {}", e);
        return;
    }

    if !PathBuf::from(default_config_path).exists() {
        match File::create(default_config_path) {
            Ok(mut file) => {
                let content = confique::toml::template::<Conf>(FormatOptions::default());
                if let Err(e) = file.write_all(content.as_bytes()) {
                    eprintln!("Error writing to config file: {}", e);
                } else {
                    println!("A new config has been created at `{}`", default_config_path);
                }
            }
            Err(e) => eprintln!("Error creating config file: {}", e),
        }
    } else {
        println!("Config file already exists at `{}`", default_config_path);
    }
}
