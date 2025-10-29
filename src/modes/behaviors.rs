// pub mod audio;
pub mod solid;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    core::{led_color::LedColor, strip::Strip},
    modes::behaviors::solid::SolidBhv,
};

pub trait Behavior {
    fn new(strip: Box<dyn Strip>) -> Result<impl Behavior>;

    async fn poll_next(&mut self, colors: &[LedColor]) -> Result<()>;
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum BehaviorMod {
    Solid,
}

impl BehaviorMod {
    pub async fn get_behavior(&self, strip: Box<dyn Strip>) -> Result<impl Behavior> {
        match self {
            BehaviorMod::Solid => SolidBhv::new(strip),
        }
    }
}
