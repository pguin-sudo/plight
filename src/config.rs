use confique::{toml::FormatOptions, Config};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use lazy_static::lazy_static;

use crate::modes::cava_wall_dcol::CavaWallDcolModConf;
use crate::modes::color::ColorModConf;
use crate::modes::screen::ScreenModConf;
use crate::modes::wallpaper::WallpaperModConf;
use crate::modes::Mode;

lazy_static! {
    // ? Maybe I should use RwLock there
    pub static ref CONFIG: Conf = Conf::new();
}

#[derive(Config)]
pub struct Conf {
    #[config(default = "Wallpaper")]
    pub mode: Mode,

    // Strip configuration
    #[config(nested)]
    pub strip: StripConf,

    // Several PLight modes config
    #[config(nested)]
    pub modes: ModesConf,
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

#[derive(Clone, Config)]
pub struct StripConf {
    #[config(default = 29)]
    pub width: usize,
    #[config(default = 15)]
    pub height: usize,
    #[config(default = 7)]
    pub bottom_gap: usize,
    // Width of coreners (pixels)
    #[config(default = 0)]
    pub corner_size_p: usize,
    #[config(default = 200)]
    pub thickness_p: usize,
    /// #[config(default = false)]
    /// pub clockwise: bool,

    #[config(default = "/dev/ttyUSB0")]
    pub serial_port: String,
    #[config(default = 115200)]
    pub baudrate: u32,

    // Tint configuration
    #[config(nested)]
    pub tint: TintConf,
}

impl StripConf {
    pub fn len(&self) -> usize {
        self.width * 2 + self.height * 2 - self.bottom_gap
    }
}

#[derive(Clone, Config)]
pub struct TintConf {
    #[config(default = "GRB")]
    pub order: String,
    #[config(default = [0.2, 0.2, 0.2])]
    pub gamma: [f32; 3],
    #[config(default = [1.0, 0.9, 0.9])]
    pub saturation: [f32; 3],
    #[config(default = [1.0, 1.0, 1.0])]
    pub brightness: [f32; 3],
}

#[derive(Config)]
pub struct ModesConf {
    #[config(nested)]
    pub color: ColorModConf,
    #[config(nested)]
    pub cava_wall_dcol: CavaWallDcolModConf,
    #[config(nested)]
    pub screen: ScreenModConf,
    #[config(nested)]
    pub wallpaper: WallpaperModConf,
}

