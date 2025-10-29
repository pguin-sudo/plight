use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;
use confique::Config;
use log::{error, info};
use pipewire::context::Context;
use pipewire::keys;
use pipewire::main_loop::MainLoop;
use pipewire::properties::properties;
use pipewire::spa::param::format::{MediaSubtype, MediaType};
use pipewire::spa::param::{format_utils, ParamType};
use pipewire::spa::pod::Pod;
use pipewire::spa::utils::Direction;
use pipewire::stream::{Stream, StreamFlags, StreamListener, StreamRef};
use tokio::sync::mpsc;

use crate::config::CONFIG;
use crate::core::led_color::LedColor;
use crate::core::strip::Strip;
use crate::modes::behaviors::Behavior;
use crate::utils::audio::calculate_sound_level;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct AudioBhvConf {
    #[config(default = [222, 155, 144])]
    pub color: [u8; 3],

    #[config(default = 0.05)]
    pub flicker: f64,
}

pub struct AudioBhv {
    strip: Box<dyn Strip>,
    audio_level_rx: Option<mpsc::UnboundedReceiver<f64>>,
}

impl AudioBhv {
    pub fn new(strip: Box<dyn Strip>) -> Result<Self> {
        let (audio_level_tx, audio_level_rx) = mpsc::unbounded_channel();

        thread::spawn(move || {
            if let Err(e) = Self::run_pipewire_loop(audio_level_tx) {
                error!("PipeWire thread error: {}", e);
            }
        });

        Ok(AudioBhv {
            strip,
            audio_level_rx: Some(audio_level_rx),
        })
    }
}

impl Behavior for AudioBhv {
    fn poll_next(&mut self, colors: &[LedColor]) -> Result<()> {
        if let Some(rx) = &mut self.audio_level_rx {
            let mut current_audio_level = 0.0;
            while let Ok(level) = rx.try_recv() {
                current_audio_level = level;
            }

            let new_colors: Vec<LedColor> = colors
                .iter()
                .map(|color| *color * current_audio_level)
                .collect();
            let _ = self.strip.set_leds(&new_colors);
        }

        Ok(())
    }
}

impl AudioBhv {
    fn run_pipewire_loop(audio_level_tx: mpsc::UnboundedSender<f64>) -> Result<()> {
        let mainloop = MainLoop::new(None)?;
        let context = Context::new(&mainloop)?;
        let core = context.connect(None)?;

        let props = properties! {
            *keys::APP_NAME => "PLight",
            *keys::FORMAT_DSP => "32 bit float mono audio",
            *keys::MEDIA_CATEGORY => "Capture",
            *keys::MEDIA_CLASS => "Stream/Input/Audio",
            *keys::MEDIA_NAME => "plight",
            *keys::MEDIA_ROLE => "Music",
            *keys::MEDIA_TYPE => "Audio",
            *keys::NODE_ALWAYS_PROCESS => "true",
            *keys::NODE_AUTOCONNECT => "true",
            *keys::NODE_NAME => "PLight",
            *keys::PRIORITY_SESSION => "9999",
        };

        let stream = Stream::new(&core, "PLight audio capture", props)?;

        stream.connect(
            Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut [],
        )?;

        let current_value = Arc::new(Mutex::new(0.0f64));

        let param_changed_cb =
            move |_listener: &StreamRef, _object_id: &mut _, id: u32, param: Option<&Pod>| {
                let Some(param) = param else { return };
                if id != ParamType::Format.as_raw() {
                    return;
                }

                let (media_type, media_subtype) = match format_utils::parse_format(param) {
                    Ok(v) => v,
                    Err(_) => return,
                };

                if media_type != MediaType::Audio || media_subtype != MediaSubtype::Raw {
                    return;
                }

                info!("Audio format configured successfully");
            };

        let process_cb = {
            let audio_level_tx = audio_level_tx.clone();
            let current_value = current_value.clone();

            move |stream: &StreamRef, _data: &mut _| {
                if let Some(mut buf) = stream.dequeue_buffer() {
                    if let Some(data) = buf.datas_mut()[0].data() {
                        let sound_level = calculate_sound_level(data);
                        let mut current = current_value.lock().unwrap();
                        let prev_sound_level = *current;
                        *current = prev_sound_level
                            + CONFIG.behavior.audio.flicker * (sound_level - prev_sound_level);

                        let _ = audio_level_tx.send(*current);
                    }
                }

                if let Err(e) = stream.flush(false) {
                    error!("Error flushing stream: {}", e);
                }
            }
        };

        let _listener: StreamListener<u32> = stream
            .add_local_listener()
            .param_changed(param_changed_cb)
            .process(process_cb)
            .register()?;

        info!("PipeWire audio capture started");

        mainloop.run();

        Ok(())
    }
}
