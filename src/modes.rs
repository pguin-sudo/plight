pub mod audio;
pub mod cava_wall_dcol;
pub mod color;
pub mod screen;
pub mod wallpaper;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{errors::Result, strip::Strip};

// ! pacman -S libxcb libxrandr dbus

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum Mode {
    Audio,
    CavaWallDcol,
    Color,
    Screen,
    Wallpaper,
}

impl Mode {
    pub async fn poll(&self, strip_arc: Arc<Strip>, strip: &mut Strip) -> Result<()> {
        println!("Polling is starting");
        match self {
            Mode::Audio => self.poll_audio(strip_arc).await,
            Mode::CavaWallDcol => self.poll_cava_wall_dcol(strip).await,
            Mode::Color => self.poll_color(strip).await,
            Mode::Screen => self.poll_screen(strip).await,
            Mode::Wallpaper => self.poll_wallpaper(strip).await,
        }
    }
}
