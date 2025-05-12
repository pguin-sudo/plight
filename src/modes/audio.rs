use confique::Config;
use image::{Pixel, Rgb};
use pipewire::context::Context;
use pipewire::keys;
use pipewire::main_loop::MainLoop;
use pipewire::properties::properties;
use pipewire::spa::param::format::{MediaSubtype, MediaType};
use pipewire::spa::param::{format_utils, ParamType};
use pipewire::spa::utils::Direction;
use pipewire::stream::{Stream, StreamFlags};
use std::str;
use std::sync::{Arc, Mutex};

use crate::config::CONFIG;
use crate::errors::Result;
use crate::modes::Mode;
use crate::strip::Strip;
use crate::utils::sound::calculate_sound_level;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct AudioModConf {
    #[config(default = [222, 155, 144])]
    pub color: [u8; 3],

    #[config(default = 0.05)]
    pub flicker: f64,
}

#[derive(Clone, Debug)]
struct AudioProcessData {
    pub current_value: Arc<Mutex<Option<f64>>>,
}

impl Mode {
    pub async fn poll_audio(&self, strip: Arc<Strip>) -> Result<()> {
        let length: usize = CONFIG.strip.len().into();
        let current_value = Arc::new(Mutex::new(None));

        let data = AudioProcessData {
            current_value: current_value.clone(),
        };

        let mainloop = MainLoop::new(None)?;
        let context = Context::new(&mainloop)?;
        let core = context.connect(None)?;

        let props = properties! {
            *keys::APP_NAME => "PLight",
            *keys::AUDIO_CHANNEL => "FL",
            *keys::AUDIO_CHANNELS => "2",
            *keys::MEDIA_TYPE => "Audio",
            *keys::MEDIA_CATEGORY => "DSP",
            *keys::MEDIA_ROLE => "Accessibility",
            *keys::NODE_NAME => "PLight",
            *keys::PORT_ALIAS => "Inputttt"
        };
        let stream = Stream::new(&core, "Plight monitor", props)?;

        stream.connect(
            Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut [],
        )?;

        let strip_arc = Arc::new(Mutex::new(strip));
        let strip_clone = strip_arc.clone();

        let _listener_l = stream
            .add_local_listener_with_user_data(data)
            .param_changed(|_, _, id, param| {
                let Some(param) = param else {
                    return;
                };
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
            })
            .process(|stream, apd| {
                if let Some(mut buf) = stream.dequeue_buffer() {
                    if let Some(data) = buf.datas_mut()[0].data() {
                        let sound_level = calculate_sound_level(data);
                        let mut current = apd.current_value.lock().unwrap();
                        let prev_sound_level = (*current).unwrap_or_default();
                        *current = Some(
                            prev_sound_level
                                + CONFIG.modes.audio.flicker.clone()
                                    * (sound_level - prev_sound_level),
                        );
                    }
                }
                stream.flush(false).unwrap();
            })
            .register()?;

        std::thread::spawn(move || {
            let base_color = Rgb::from(CONFIG.modes.audio.color);
            // TODO: Do something with error handling
            loop {
                let current_audio_level = current_value.lock().unwrap().unwrap_or_default();

                let color = base_color.map(|x| (x as f64 * current_audio_level) as u8);

                let colors = vec![color; length];
                if let Ok(strip) = strip_clone.lock() {
                    if let Err(e) = strip.set_leds(&colors) {
                        eprintln!("Thread's error setting LEDs: {}", e);
                    }
                }

                // ? Maybe we need some delay to prevent busy waiting
                // std::thread::sleep(std::time::Duration::from_millis(16));
            }
        });

        mainloop.run();

        Ok(())
    }
}
