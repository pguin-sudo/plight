use confique::Config;
use image::{open, Rgb};
use serde::{Deserialize, Serialize};
use std::str;
use tokio::process::Command;

use crate::config::CONFIG;
use crate::errors::{Error::WrongWallpaperPath, Result};
use crate::modes::Mode;
use crate::strip::Strip;
use crate::utils::{parse_image, rotate_smooth};

#[derive(Config)]
pub struct WallpaperModConf {
    #[config(default = "Swww")]
    pub engine: WallpaperEngine,

    // ! THIS IS NOT WORKING CORRECTLY
    #[config(default = 0)]
    pub rotation_speed: f32,
}

impl Mode {
    pub async fn poll_wallpaper(&self, strip: &mut Strip) -> Result<()> {
        let mut command;
        let path_to_wallpaper = match CONFIG.modes.wallpaper.engine {
            WallpaperEngine::Swww => {
                command = Command::new("swww");
                command.args(["query"])
            }
        };

        let image_prefix = "image: ";
        // ? Maybe I should use box there
        let mut colors: Vec<Rgb<u8>> = Vec::new();
        let mut prev_output_str: String = "".into();

        loop {
            let output = path_to_wallpaper.output().await?;

            let output_str = str::from_utf8(&output.stdout)?;

            if output_str == prev_output_str {
                if CONFIG.modes.wallpaper.rotation_speed != 0.0 {
                    colors = rotate_smooth(&mut colors, CONFIG.modes.wallpaper.rotation_speed);
                    strip.set_leds(&colors)?;
                }
                continue;
            }

            prev_output_str = output_str.to_string();

            match output_str.split_once(image_prefix) {
                Some((_, image_path)) => {
                    let image_path = &image_path.replace("\n", "");

                    let image = open(image_path)?.into_rgb8();

                    colors = parse_image(&image).await;
                    strip.set_leds(&colors)?;
                }
                None => {
                    return Err(WrongWallpaperPath {
                        given: output_str.to_string(),
                    })
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum WallpaperEngine {
    Swww,
}
