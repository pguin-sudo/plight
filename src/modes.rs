use rgb::RGB8;
use serde::{Deserialize, Serialize};

pub mod cava_wall_dcol;
pub mod color;
pub mod wallpaper;

use crate::config::Conf;

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum Mode {
    CavaWallDcol,
    Color,
    Wallpaper,
}

impl Mode {
    pub async fn poll<F>(&self, config: &Conf, draw: F)
    where
        F: FnMut(&[RGB8]),
    {
        match self {
            Mode::CavaWallDcol => self.poll_cava_wall_dcol(config, draw).await,
            Mode::Color => self.poll_color(config, draw).await,
            Mode::Wallpaper => self.poll_wallpaper(config, draw).await,
        };
    }
}
