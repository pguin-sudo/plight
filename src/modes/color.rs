use confique::Config;
use rgb::RGB8;
use tokio::time::{sleep, Duration};

use crate::{config::Conf, modes::Mode};

#[derive(Config)]
pub struct ColorModConf {
    // Update rate in milliseconds
    #[config(default = 32)]
    pub update_rate: u64,

    #[config(default = [192, 168, 31])]
    pub color: [u8; 3],
}

impl Mode {
    pub async fn poll_color<F>(&self, config: &Conf, mut draw: F)
    where
        F: FnMut(&[RGB8]),
    {
        let length: usize = config.strip.len().into();
        loop {
            let [r, g, b] = config.modes.color.color;
            let colors = vec![RGB8::new(r, g, b); length];
            draw(&colors);
            sleep(Duration::from_millis(config.modes.color.update_rate)).await;
        }
    }
}
