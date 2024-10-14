use rgb::RGB8;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

use crate::config::Conf;

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub enum Mode {
    Color,
}

impl Mode {
    pub fn process(&self, config: &Conf) -> Vec<RGB8> {
        match self {
            Self::Color => {
                let [r, g, b] = config.modes.color.color;
                vec![RGB8::new(r, g, b); config.strip.length().into()]
            }
        }
    }
}
