use std::process::Command;
use std::str;

use anyhow::Result;
use confique::Config;
use image::open;
use serde::{Deserialize, Serialize};

use crate::config::CONFIG;
use crate::core::led_sequence::LedSequence;
use crate::errors::PLightError::WrongWallpaperPath;
use crate::modes::sources::Source;
use crate::utils::image_processing::parse_image;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct WallpaperSrcConf {
    #[config(default = "Swww")]
    pub engine: WallpaperEngine,
}

pub struct WallpaperSrc {
    wallpaper_command: Command,
    prev_output_str: String,
    image_prefix: &'static str,
    colors: LedSequence,
}

impl WallpaperSrc {
    pub fn new() -> Result<Self> {
        let wallpaper_command = match CONFIG.source.wallpaper.engine {
            WallpaperEngine::Swww => {
                let mut command = Command::new("swww");
                command.args(["query"]);
                command
            }
        };

        let image_prefix = "image: ";
        let colors = LedSequence::new(CONFIG.strip.len());

        let prev_output_str: String = "".into();

        Ok(WallpaperSrc {
            wallpaper_command,
            prev_output_str,
            image_prefix,
            colors,
        })
    }
}

impl Source for WallpaperSrc {
    fn poll_next(&mut self, led_sequence: &mut LedSequence) -> Result<()> {
        let output = self.wallpaper_command.output()?;

        let output_str = str::from_utf8(&output.stdout)?;

        if output_str == self.prev_output_str {
            led_sequence.set_sequence(self.colors.clone());
            return Ok(());
        }

        self.prev_output_str = output_str.to_string();

        match output_str.split_once(self.image_prefix) {
            Some((_, image_path)) => {
                let image_path = &image_path.replace("\n", "");

                let image = open(image_path)?.into_rgb8();

                parse_image(&image, &mut self.colors);

                led_sequence.set_sequence(self.colors.clone());
                Ok(())
            }
            None => Err(WrongWallpaperPath {
                given: output_str.to_string(),
            }
            .into()),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum WallpaperEngine {
    Swww,
}
