use confique::Config;
use image::Rgb;
use std::sync::Mutex;
use std::str;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait}; 
use cpal::{default_host, StreamConfig};
use hound::{WavWriter, WavSpec};

use crate::config::CONFIG;
use crate::strip::{SetLedsError, Strip};
use crate::modes::Mode;

#[derive(Config)]
pub struct MusicModConf {}

impl Mode {
    pub async fn poll_music(&self, strip: Mutex<Strip>) -> Result<(), SetLedsError> {
        let host = default_host();
        let device = host.default_input_device().expect("No input device available");

        let input_config = device.default_input_config().expect("Failed to get default input config");

        let spec = WavSpec {
            channels: input_config.channels(),
            sample_rate: input_config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let stream_config: StreamConfig = input_config.into();
        
        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
            let length = CONFIG.strip.len();
                for &sample in data {
                    let colors = vec![Rgb::<u8>::from([sample as u8, 0, 0]); length];
                    strip.lock().unwrap().set_leds(&colors);
                }
            },
            |err| eprintln!("Error occurred on stream: {:?}", err),
            None 
        ).expect("Failed to create input stream");

        stream.play().expect("Failed to play stream");

        loop {
            
        }
    }
}
