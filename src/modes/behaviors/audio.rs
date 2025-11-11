use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use anyhow::Result;
use confique::Config;
use log::{error, info, trace, warn};
use pipewire::context::Context;
use pipewire::keys;
use pipewire::main_loop::MainLoop;
use pipewire::properties::properties;
use pipewire::spa::param::audio::{AudioFormat, AudioInfoRaw};
use pipewire::spa::param::format::{FormatProperties, MediaSubtype, MediaType};
use pipewire::spa::param::{format_utils, ParamType};
use pipewire::spa::pod::serialize::PodSerializer;
use pipewire::spa::pod::{object, property, Pod, Value};
use pipewire::spa::utils::{Direction, SpaTypes};
use pipewire::stream::{Stream, StreamFlags, StreamListener, StreamRef};
use unit_interval::UnitInterval;

use crate::core::led_sequence::LedSequence;
use crate::core::strip::Strip;
use crate::modes::behaviors::Behavior;
use crate::utils::audio::{make_get_sound_level_closure, smooth_audio_level};

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct AudioBhvConf {
    /// Attack for smooth algorithm
    #[config(default = 0.07)]
    pub attack: f64,

    /// Decay for smooth algorithm
    #[config(default = 0.07)]
    pub decay: f64,

    /// At such a volume and below, there will be no color
    #[config(default = -80)]
    pub min_level_db: f64,

    /// At this volume, the color will not be changed
    #[config(default = -15)]
    pub max_level_db: f64,
}

pub struct AudioBhv {
    strip: Box<dyn Strip>,
    audio_level_rx: Option<mpsc::Receiver<(UnitInterval<f64>, UnitInterval<f64>)>>,
}

impl AudioBhv {
    pub fn new(strip: Box<dyn Strip>) -> Result<Self> {
        let (audio_level_tx, audio_level_rx) = mpsc::channel();

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
    fn poll_next(&mut self, colors: &LedSequence) -> Result<()> {
        if let Some(rx) = &mut self.audio_level_rx {
            let mut current_audio_level = (UnitInterval::zero(), UnitInterval::zero());

            while let Ok(level) = rx.try_recv() {
                current_audio_level = level;
            }

            let _ = self
                .strip
                .set_leds(&colors.clone().adjusted_halfs_value(current_audio_level));
        }

        Ok(())
    }
}

impl AudioBhv {
    fn run_pipewire_loop(
        audio_level_tx: mpsc::Sender<(UnitInterval<f64>, UnitInterval<f64>)>,
    ) -> Result<()> {
        let mainloop = MainLoop::new(None)?;
        let context = Context::new(&mainloop)?;
        let core = context.connect(None)?;

        let props = properties! {
            *keys::FORMAT_DSP => "32 bit float stereo audio",
            *keys::MEDIA_CATEGORY => "Capture",
            *keys::MEDIA_CLASS => "Stream/Input/Audio",
            *keys::MEDIA_NAME => "Audio",
            *keys::MEDIA_ROLE => "Music",
            *keys::MEDIA_TYPE => "Audio",
            *keys::NODE_ALWAYS_PROCESS => "true",
            *keys::NODE_AUTOCONNECT => "true",
            *keys::NODE_NAME => "PLight",
            *keys::PRIORITY_SESSION => "9999",
            *keys::STREAM_CAPTURE_SINK => "true",
        };

        let stream = Stream::new(&core, "audio-capture", props)?;

        let obj = object!(
            SpaTypes::ObjectParamFormat,
            ParamType::EnumFormat,
            property!(FormatProperties::MediaType, Id, MediaType::Audio),
            property!(FormatProperties::MediaSubtype, Id, MediaSubtype::Raw),
            property!(FormatProperties::AudioFormat, Id, AudioFormat::F32LE),
            property!(FormatProperties::AudioRate, Int, 48000),
            property!(FormatProperties::AudioChannels, Int, 2),
        );

        let values: Vec<u8> =
            PodSerializer::serialize(std::io::Cursor::new(Vec::new()), &Value::Object(obj))
                .unwrap()
                .0
                .into_inner();

        let pod = Pod::from_bytes(&values).unwrap();

        let mut params = [pod];

        stream.connect(
            Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut params,
        )?;

        let current_value = Arc::new(Mutex::new((UnitInterval::zero(), UnitInterval::zero())));

        let get_sound_level = Arc::new(Mutex::new(Box::new(make_get_sound_level_closure(
            AudioInfoRaw::default(),
        ))));

        let cb_get_sound_level = get_sound_level.clone();

        let param_changed_cb = {
            move |_listener: &StreamRef, _object_id: &mut _, id: u32, param: Option<&Pod>| {
                let Some(param) = param else {
                    info!("ParamChanged: No param provided (likely cleared)");
                    return;
                };

                if id != ParamType::Format.as_raw() {
                    trace!("ParamChanged: Ignoring non-Format param (id={})", id);
                    return;
                }

                let (media_type, media_subtype) = match format_utils::parse_format(param) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("ParamChanged: Failed to parse format: {:?}", e);
                        return;
                    }
                };

                if media_type != MediaType::Audio {
                    warn!("ParamChanged: Expected Audio, got {:?}", media_type);
                    return;
                }
                if media_subtype != MediaSubtype::Raw {
                    warn!("ParamChanged: Expected Raw, got {:?}", media_subtype);
                    return;
                }

                let mut audio_info = AudioInfoRaw::new();

                match audio_info.parse(param) {
                    Ok(info) => info,
                    Err(e) => {
                        error!("ParamChanged: Failed to parse AudioInfoRaw: {:?}", e);
                        return;
                    }
                };

                let mut guard = cb_get_sound_level.lock().unwrap();
                *guard = Box::new(make_get_sound_level_closure(audio_info));

                info!(
                    "Audio format negotiated: {} Hz | {} ch | format: {:?}",
                    audio_info.rate(),
                    audio_info.channels(),
                    audio_info.format(),
                );
            }
        };

        let process_cb = {
            let audio_level_tx = audio_level_tx.clone();
            let current_value = current_value.clone();

            move |stream: &StreamRef, _data: &mut _| {
                if let Some(mut buf) = stream.dequeue_buffer() {
                    let data = &mut buf.datas_mut()[0];
                    let mut current = current_value.lock().unwrap();
                    let prev_sound_level = *current;
                    if let Ok(sound_level) = get_sound_level.lock().unwrap()(data) {
                        *current = (
                            smooth_audio_level(sound_level.0, prev_sound_level.0),
                            smooth_audio_level(sound_level.1, prev_sound_level.1),
                        );

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
