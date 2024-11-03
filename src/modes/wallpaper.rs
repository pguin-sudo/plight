use confique::Config;
use image::open;
use rgb::RGB8;
use serde::{Deserialize, Serialize};
use std::str;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use crate::utils::{average_color, parse_image};
use crate::{config::Conf, modes::Mode};

#[derive(Config)]
pub struct WallpaperModConf {
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,

    #[config(default = "Swww")]
    pub engine: WallpaperEngine,
}

impl Mode {
    pub async fn poll_wallpaper<F>(&self, config: &Conf, mut draw: F)
    where
        F: FnMut(&[RGB8]),
    {
        let mut command;
        let path_to_wallpaper = match config.modes.wallpaper.engine {
            WallpaperEngine::Swww => {
                command = Command::new("swww");
                command.args(["query"])
            }
        };

        let image_prefix = "image: ";
        let mut previous_output_str: String = "".into();

        loop {
            let output = path_to_wallpaper
                .output()
                .await
                .expect("Error while loading image");

            let output_str = str::from_utf8(&output.stdout).expect("Error while loading image");

            if output_str == previous_output_str {
                continue;
            }

            previous_output_str = (output_str).to_string();

            let image_path = match output_str.split_once(image_prefix) {
                Some((_, path)) => path.replace("\n", ""),
                None => {
                    eprintln!("Error: 'image: ' not found in output");
                    continue;
                }
            };

            let image = open(image_path)
                .expect("Error while opening image")
                .into_rgb8();

            draw(&parse_image(&image, average_color, &config.strip));

            sleep(Duration::from_millis(config.modes.wallpaper.update_rate)).await;
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum WallpaperEngine {
    Swww,
}
