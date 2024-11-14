use confique::Config;
use image::{Pixel, Rgb};
use std::sync::{Arc, Mutex, RwLock};
use std::str;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait}; 
use cpal::{default_host, StreamConfig};

use crate::config::CONFIG;
use crate::strip::{SetLedsError, Strip};
use crate::modes::Mode;

#[derive(Config)]
pub struct MusicModConf {
    #[config(default = 1.5)]
    coefficient: f32,
}

impl Mode {
    pub async fn poll_music(&self, strip: &Mutex<Strip>) -> Result<(), SetLedsError> {
        let host = default_host();
        let device = host.default_input_device().expect("No input device available");

        let input_config = device.default_input_config().expect("Failed to get default input config");
        let stream_config: StreamConfig = input_config.into();

        let sample  = Arc::new(RwLock::new(0_f32));

        let stream_sample = Arc::clone(&sample);        
        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut s = stream_sample.write().unwrap();
                *s = data.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0_f32);
            },
            |err| eprintln!("Error occurred on stream: {:?}", err),
            None 
        ).expect("Failed to create input stream");

        stream.play().expect("Failed to play stream");

        let length = CONFIG.strip.len();
        
        let mut prev_color = *Rgb::from_slice(&[202_u8, 126_u8, 137_u8]);

        loop {
            let mut color  = *Rgb::from_slice(&[202_u8, 126_u8, 137_u8]);

            color.apply(|x| {
                (x as f32 * CONFIG.modes.music.coefficient * *sample.read().unwrap()).round() as u8 
            });

            let lerp_factor = 0.1;
            prev_color = *Rgb::from_slice(&[
                lerp(prev_color[0] as f32, color[0] as f32, lerp_factor) as u8,
                lerp(prev_color[1] as f32, color[1] as f32, lerp_factor) as u8,
                lerp(prev_color[2] as f32, color[2] as f32, lerp_factor) as u8,
            ]);

            let colors = vec![prev_color; length];
            strip.lock().unwrap().set_leds(&colors)?;
        }
    }
}

fn lerp(start: f32, end: f32, factor: f32) -> f32 {
    start + factor * (end - start)
}            



 
