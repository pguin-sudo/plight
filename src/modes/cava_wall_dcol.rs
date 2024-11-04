use confique::FileFormat;
use confique::{Config, File};
use image::Rgb;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use crate::{config::Conf, modes::Mode, utils::hex_to_rgb};

#[derive(Config)]
pub struct CavaWallDcolModConf {
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,

    #[config(default = "/home/pguin/.config/cava/Wall-Dcol")]
    pub path_to_dcol: PathBuf,
}

impl Mode {
    pub async fn poll_cava_wall_dcol<F>(&self, config: &Conf, mut draw: F)
    where
        F: FnMut(&[Rgb<u8>]),
    {
        const GRADIENT_LENGTH: usize = 7;

        loop {
            let cava_gradients = CavaGradientsConf::from_partial(
                File::with_format(&config.modes.cava_wall_dcol.path_to_dcol, FileFormat::Toml)
                    .load()
                    .expect("Error loading config"),
            )
            .expect("Error loading config");

            let gradient_colors: [Rgb<u8>; GRADIENT_LENGTH] = [
                hex_to_rgb(&cava_gradients.color.gradient_color_8),
                hex_to_rgb(&cava_gradients.color.gradient_color_7),
                hex_to_rgb(&cava_gradients.color.gradient_color_6),
                hex_to_rgb(&cava_gradients.color.gradient_color_5),
                hex_to_rgb(&cava_gradients.color.gradient_color_2),
                hex_to_rgb(&cava_gradients.color.gradient_color_2),
                hex_to_rgb(&cava_gradients.color.gradient_color_2),
            ];

            let mut colors = Vec::<Rgb<u8>>::with_capacity(
                2 * (config.strip.width + config.strip.height) - config.strip.bottom_gap,
            );

            // Bottom right
            colors.extend_from_slice(&vec![
                gradient_colors[GRADIENT_LENGTH - 1];
                (config.strip.width - config.strip.bottom_gap) / 2
            ]);

            // Right
            for i in 0..config.strip.height {
                let color_index = (i * GRADIENT_LENGTH / config.strip.height) % GRADIENT_LENGTH;
                colors.push(gradient_colors[GRADIENT_LENGTH - 1 - color_index]);
            }

            // Top
            colors.extend_from_slice(&vec![gradient_colors[0]; config.strip.width.into()]);

            // Left
            for i in 0..config.strip.height {
                let color_index = (i * GRADIENT_LENGTH / config.strip.height) % GRADIENT_LENGTH;
                colors.push(gradient_colors[color_index]);
            }

            // Bottom left
            colors.extend_from_slice(&vec![
                gradient_colors[GRADIENT_LENGTH - 1];
                (config.strip.width - config.strip.bottom_gap) / 2
            ]);

            draw(&colors);
            sleep(Duration::from_millis(
                config.modes.cava_wall_dcol.update_rate,
            ))
            .await;
        }
    }
}

#[derive(Config)]
pub struct CavaGradientsConf {
    #[config(nested)]
    color: CavaGradientsColorConf,
}

#[derive(Config)]
pub struct CavaGradientsColorConf {
    // gradient: u8,
    // gradient_count: u8,
    // gradient_color_1: String,
    gradient_color_2: String,
    // gradient_color_3: String,
    // gradient_color_4: String,
    gradient_color_5: String,
    gradient_color_6: String,
    gradient_color_7: String,
    gradient_color_8: String,
}
