use confique::Config;
use image::Rgb;
use pipewire::context::Context;
use pipewire::keys;
use pipewire::main_loop::MainLoop;
use pipewire::properties::properties;
use pipewire::spa::param::format::{MediaSubtype, MediaType};
use pipewire::spa::param::{format_utils, ParamType};
use pipewire::spa::utils::Direction;
use pipewire::stream::{Stream, StreamFlags};
use std::str;

use crate::config::CONFIG;
use crate::errors::Result;
use crate::modes::Mode;
use crate::strip::Strip;
use crate::utils::time;


#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct AudioModConf {
    #[config(default = [222, 155, 144])]
    pub color: [u8; 3],

    #[config(default = 10)]
    pub timer_length: u64,
}

impl Mode {
    pub async fn poll_audio(&self, strip: &mut Strip) -> Result<()> {
        let length: usize = CONFIG.strip.len().into();
        let prev_color = Rgb::from([0_u8, 0_u8, 0_u8]);

        let data = AudioProcessData {
            timer: time::Timer::new(CONFIG.modes.audio.timer_length),
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

        // TODO: There is only one input, change it
        stream.connect(
            Direction::Input,
            Some(66),
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut [],
        )?;

        let _listener_l = stream
            .add_local_listener_with_user_data(data)
            .param_changed(|_, user_data, id, param| {
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

                println!("user data:{:?}", user_data);
            })
            .process(|stream, apd| {
                if let Some(mut buf) = stream.dequeue_buffer() {
                    if let Some(data) = buf.datas_mut()[0].data() {
                        let s: usize = data
                            .iter()
                            // checked_ilog(10).unwrap_or(0)
                            .map(|x| (*x).pow(2) as usize)
                            .sum();
                        let f = apd.timer.update_value(s);
                        println!("{:?}", f);
                    }
                }
                ("{:?}", stream.flush(false).unwrap());
            })
            .register()?;

        // let strip_lock = RwLock::new(strip);
        // thread::spawn(|| {
        //     loop {
        //         // let mut buffer = vec![0; 1024];
        //         let color = Rgb::from(CONFIG.modes.audio.color);

        //         if prev_color == color {
        //             continue;
        //         }

        //         prev_color = color;

        //         let colors = vec![color; length];
        //         strip_lock.read().unwrap().set_leds(&colors).unwrap();
        //     }
        // });

        mainloop.run();

        Ok(())
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
struct AudioProcessData {
    pub timer: time::Timer,
}
