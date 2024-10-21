use crate::config::Conf;
use rgb::RGB8;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum Mode {
    Color,
}

impl Mode {
    pub async fn poll<F>(&self, config: &Conf, draw: F)
    where
        F: FnMut(Vec<RGB8>),
    {
        match self {
            Mode::Color => self.poll_color(config, draw).await,
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
}
