use image::Rgb;
use serde::{Deserialize, Serialize};

pub mod cava_wall_dcol;
pub mod color;
pub mod screen;
pub mod wallpaper;

use crate::config::Conf;

// ! pacman -S libxcb libxrandr dbus

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum Mode {
    CavaWallDcol,
    Color,
    Screen,
    Wallpaper,
}

impl Mode {
    pub async fn poll<F>(&self, config: &Conf, draw: F)
    where
        F: FnMut(&[Rgb<u8>]),
    {
        match self {
            Mode::CavaWallDcol => self.poll_cava_wall_dcol(config, draw).await,
            Mode::Color => self.poll_color(config, draw).await,
            Mode::Screen => self.poll_screen(config, draw).await,
            Mode::Wallpaper => self.poll_wallpaper(config, draw).await,
        };
    }
}
