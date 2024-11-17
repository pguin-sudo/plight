use confique::Config;
use image::{open, Rgb};
use serde::{Deserialize, Serialize};
use std::str;
use std::sync::Mutex;
use tokio::process::Command;

use crate::config::CONFIG;
use crate::modes::Mode;
use crate::strip::{SetLedsError, Strip};
use crate::utils::{average_color, parse_image, rotate_smooth};

#[derive(Config)]
pub struct WallpaperModConf {
    #[config(default = "Swww")]
    pub engine: WallpaperEngine,

    // ! THIS IS NOT WORKING CORRECTLY
    #[config(default = 0)]
    pub rotation_speed: f32,
}

impl Mode {
    pub async fn poll_wallpaper(&self, strip: &Mutex<Strip>) -> Result<(), SetLedsError> {
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
            let output = path_to_wallpaper
                .output()
                .await
                .expect("Error while loading image");

            let output_str = str::from_utf8(&output.stdout).expect("Error while loading image");

            if output_str == prev_output_str {
                if CONFIG.modes.wallpaper.rotation_speed != 0.0 {
                    colors = rotate_smooth(&mut colors, CONFIG.modes.wallpaper.rotation_speed);
                    strip.lock().unwrap().set_leds(&colors)?;
                    continue;
                }

                continue;
            }

            prev_output_str = (output_str).to_string();

            let image_path = match output_str.split_once(image_prefix) {
                Some((_, path)) => path.replace("\n", ""),
                _ => {
                    eprintln!("Error: 'image: ' not found in output {}", output_str);
                    continue;
                }
            };

            let image = open(image_path)
                .expect("Error while opening image")
                .into_rgb8();

            colors = parse_image(&image, average_color).await;
            strip.lock().unwrap().set_leds(&colors)?;
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum WallpaperEngine {
    Swww,
}
