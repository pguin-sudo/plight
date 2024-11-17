pub mod cava_wall_dcol;
pub mod color;
pub mod screen;
pub mod wallpaper;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::{errors::Result, strip::Strip};

// ! pacman -S libxcb libxrandr dbus

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum Mode {
    CavaWallDcol,
    Color,
    Screen,
    Wallpaper,
}

impl Mode {
    pub async fn poll(&self, strip: &Mutex<Strip>) -> Result<()> {
        match self {
            Mode::CavaWallDcol => self.poll_cava_wall_dcol(strip).await,
            Mode::Color => self.poll_color(strip).await,
            Mode::Screen => self.poll_screen(strip).await,
            Mode::Wallpaper => self.poll_wallpaper(strip).await,
        }
    }
}
