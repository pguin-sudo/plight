use confique::Config;
use image::{open, Rgb};
use serde::{Deserialize, Serialize};
use std::str;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use crate::config::CONFIG;
use crate::utils::{average_color, parse_image, rotate_smooth};
use crate::modes::Mode;

#[derive(Config)]
pub struct WallpaperModConf {
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,

    #[config(default = "Swww")]
    pub engine: WallpaperEngine,

    // ! THIS IS NOT WORKING CORRECTLY
    #[config(default = 0)]
    pub rotation_speed: f32,
}

impl Mode {
    pub async fn poll_wallpaper<F>(&self, mut draw: F)
    where
        F: FnMut(&[Rgb<u8>]),
    {
        let mut command;
        let path_to_wallpaper = match CONFIG.read().await.modes.wallpaper.engine {
            WallpaperEngine::Swww => {
                command = Command::new("swww");
                command.args(["query"])
            }
        };

        let image_prefix = "image: ";
        // ? Maybe I should use box there
        let mut colors: Vec<Rgb<u8>> = Vec::new();
        let mut previous_output_str: String = "".into();

        loop {
            let output = path_to_wallpaper
                .output()
                .await
                .expect("Error while loading image");

            let output_str = str::from_utf8(&output.stdout).expect("Error while loading image");

            if output_str == previous_output_str {
                if CONFIG.read().await.modes.wallpaper.rotation_speed != 0.0 {
                    colors = rotate_smooth(&mut colors, CONFIG.read().await.modes.wallpaper.rotation_speed);
                    draw(&colors);
                    sleep(Duration::from_millis(CONFIG.read().await.modes.wallpaper.update_rate)).await;
                    continue;
                }

                sleep(Duration::from_millis(CONFIG.read().await.modes.wallpaper.update_rate)).await;
                continue;
            }

            previous_output_str = (output_str).to_string();

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
            draw(&colors);

            sleep(Duration::from_millis(CONFIG.read().await.modes.wallpaper.update_rate)).await;
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum WallpaperEngine {
    Swww,
}
