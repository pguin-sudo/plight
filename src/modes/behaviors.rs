pub mod audio;
pub mod solid;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    core::{led_sequence::LedSequence, strip::Strip},
    modes::behaviors::{audio::AudioBhv, solid::SolidBhv},
};

pub trait Behavior {
    fn poll_next(&mut self, colors: &LedSequence) -> Result<()>;
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub enum BehaviorMod {
    Audio,
    Solid,
}

impl BehaviorMod {
    pub fn get_behavior(&self, strip: Box<dyn Strip>) -> Result<Box<dyn Behavior>> {
        match self {
            BehaviorMod::Audio => Ok(Box::new(AudioBhv::new(strip)?)),
            BehaviorMod::Solid => Ok(Box::new(SolidBhv::new(strip)?)),
        }
    }
}
