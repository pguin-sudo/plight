use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use confique::{toml::FormatOptions, Config};
use lazy_static::lazy_static;
use serde::Deserialize;
use serde::Serialize;

use anyhow::Result;

use crate::modes::behaviors::solid::SolidBhvConf;
use crate::modes::behaviors::BehaviorMod;
use crate::modes::sources::color::ColorSrcConf;
use crate::modes::sources::SourceMod;

lazy_static! {
    // ? Maybe I should use RwLock there
    pub static ref CONFIG: Conf = Conf::new().unwrap();
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct Conf {
    // Global things
    #[config(nested)]
    pub global: GlobalConf,

    // Strip configuration
    #[config(nested)]
    pub strip: StripConf,

    #[config(default = "Color")]
    pub source: SourceMod,

    // Several PLight color source configuration
    #[config(nested)]
    pub sources: SourcesConf,

    #[config(default = "Solid")]
    pub behavior: BehaviorMod,

    // Several PLight behavior configuration
    #[config(nested)]
    pub modes: BehaviorsConf,
}

impl Conf {
    pub fn new() -> Result<Conf> {
        let default_config_path = get_default_config_path()?;
        create_new_config(&default_config_path)?;
        Ok(Conf::from_file(default_config_path)?)
    }
}

fn get_default_config_path() -> Result<String> {
    let home_dir = env::var("HOME")?;
    Ok(format!("{}/.config/plight/config.toml", home_dir))
}

fn create_new_config(default_config_path: &str) -> Result<()> {
    let binding = PathBuf::from(default_config_path);
    let parent_dir = binding.parent().unwrap();
    fs::create_dir_all(parent_dir)?;

    if !PathBuf::from(default_config_path).exists() {
        let mut file = File::create(default_config_path)?;
        let content = confique::toml::template::<Conf>(FormatOptions::default());
        file.write_all(content.as_bytes())?;
    }
    Ok(())
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct GlobalConf {
    #[config(default = "Average")]
    pub parse_mode: ParseMode,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum ParseMode {
    Average,
    Median,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
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

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
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

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct SourcesConf {
    #[config(nested)]
    pub color: ColorSrcConf,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct BehaviorsConf {
    #[config(nested)]
    pub solid: SolidBhvConf,
}
