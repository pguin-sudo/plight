use crate::{config::Conf, utils::hex_to_rgb};
use confique::{Config, File, FileFormat};
use rgb::RGB8;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum Mode {
    Color,
    CavaWallDcol,
}

impl Mode {
    pub async fn poll<F>(&self, config: &Conf, draw: F)
    where
        F: FnMut(Vec<RGB8>),
    {
        match self {
            Mode::Color => self.poll_color(config, draw).await,
            Mode::CavaWallDcol => self.poll_cava_wall_dcol(config, draw).await,
        };
    }

    async fn poll_color<F>(&self, config: &Conf, mut draw: F)
    where
        F: FnMut(Vec<RGB8>),
    {
        let length: usize = config.strip.length().into();
        loop {
            let [r, g, b] = config.modes.color.color;
            let colors = vec![RGB8::new(r, g, b); length];
            draw(colors);
            sleep(Duration::from_millis(config.update_rate)).await;
        }
    }

    async fn poll_cava_wall_dcol<F>(&self, config: &Conf, mut draw: F)
    where
        F: FnMut(Vec<RGB8>),
    {
        let width = config.strip.width;
        let height = config.strip.hight;
        let bottom_gap = config.strip.bottom_gap;
        let update_rate = config.update_rate;

        const GRADIENT_LENGTH: usize = 7;

        loop {
            let cava_gradients = CavaGradientsConf::from_partial(
                File::with_format(&config.modes.cava_wall_dcol.path_to_dcol, FileFormat::Toml)
                    .load()
                    .expect("Error loading config"),
            )
            .expect("Error loading config");

            // Cache the gradient colors outside the loop
            let gradient_colors: [rgb::Rgb<u8>; GRADIENT_LENGTH] = [
                hex_to_rgb(&cava_gradients.color.gradient_color_8),
                hex_to_rgb(&cava_gradients.color.gradient_color_7),
                hex_to_rgb(&cava_gradients.color.gradient_color_6),
                hex_to_rgb(&cava_gradients.color.gradient_color_5),
                hex_to_rgb(&cava_gradients.color.gradient_color_2),
                hex_to_rgb(&cava_gradients.color.gradient_color_2),
                hex_to_rgb(&cava_gradients.color.gradient_color_2),
            ];

            let mut colors = Vec::<RGB8>::with_capacity((2 * (width + height)).into());

            // Bottom
            colors.extend_from_slice(&vec![
                gradient_colors[GRADIENT_LENGTH - 1];
                ((width - bottom_gap) / 2).into()
            ]);

            // Right / Left (two sides)
            for i in 0..height {
                let color_index = (i * GRADIENT_LENGTH as u8 / height) % GRADIENT_LENGTH as u8;
                colors.push(gradient_colors[GRADIENT_LENGTH - 1 - color_index as usize]);
            }

            // Top
            colors.extend_from_slice(&vec![gradient_colors[0]; width.into()]);

            // Left / Right (two sides)
            for i in 0..height {
                let color_index = (i * GRADIENT_LENGTH as u8 / height) % GRADIENT_LENGTH as u8;
                colors.push(gradient_colors[color_index as usize]);
            }

            // Bottom
            colors.extend_from_slice(&vec![
                gradient_colors[GRADIENT_LENGTH - 1];
                ((width - bottom_gap) / 2).into()
            ]);

            draw(colors);
            sleep(Duration::from_millis(update_rate)).await;
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
