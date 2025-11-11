use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use anyhow::Result;
use confique::Config;
use image::RgbImage;
use log::{error, info, trace, warn};
use pipewire::context::Context;
use pipewire::keys;
use pipewire::main_loop::MainLoop;
use pipewire::properties::properties;
use pipewire::spa::param::format::{FormatProperties, MediaSubtype, MediaType};
use pipewire::spa::param::video::VideoInfoRaw;
use pipewire::spa::param::{format_utils, ParamType};
use pipewire::spa::pod::serialize::PodSerializer;
use pipewire::spa::pod::{object, property, Pod, Value};
use pipewire::spa::utils::{Direction, SpaTypes};
use pipewire::stream::{Stream, StreamFlags, StreamListener, StreamRef};

use crate::core::led_sequence::LedSequence;
use crate::modes::sources::Source;
use crate::utils::image_processing::parse_image;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct ScreenSrcConf {}

pub struct ScreenSrc {
    colors_rx: Option<mpsc::Receiver<LedSequence>>,
}

impl ScreenSrc {
    pub fn new() -> Result<Self> {
        let (colors_tx, colors_rx) = mpsc::channel();

        thread::spawn(move || {
            if let Err(e) = Self::run_pipewire_loop(colors_tx) {
                error!("PipeWire thread error: {}", e);
            }
        });

        Ok(ScreenSrc {
            colors_rx: Some(colors_rx),
        })
    }
}

impl Source for ScreenSrc {
    fn poll_next(&mut self, led_sequence: &mut LedSequence) -> Result<()> {
        if let Some(rx) = &mut self.colors_rx {
            while let Ok(colors) = rx.try_recv() {
                led_sequence.set_sequence(colors);
            }
        }
        Ok(())
    }
}

impl ScreenSrc {
    fn run_pipewire_loop(colors_tx: mpsc::Sender<LedSequence>) -> Result<()> {
        let mainloop = MainLoop::new(None)?;
        let context = Context::new(&mainloop)?;
        let core = context.connect(None)?;

        let props = properties! {
            *keys::APP_NAME => "PLight",
            //*keys::FORMAT_DSP => "32 bit float stereo audio",
            *keys::MEDIA_CATEGORY => "Capture",
            //*keys::MEDIA_CLASS => "Stream/Input/Image",
            *keys::MEDIA_NAME => "plight",
            //*keys::MEDIA_ROLE => "Music",
            //*keys::MEDIA_TYPE => "Audio",
            *keys::NODE_ALWAYS_PROCESS => "true",
            *keys::NODE_AUTOCONNECT => "true",
            *keys::NODE_NAME => "PLight",
            *keys::PRIORITY_SESSION => "9999",
            *keys::STREAM_CAPTURE_SINK => "true",
        };

        let stream = Stream::new(&core, "PLight audio capture", props)?;

        let obj = object!(
            SpaTypes::ObjectParamFormat,
            ParamType::EnumFormat,
            property!(FormatProperties::MediaType, Id, MediaType::Video),
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

        let current_value = Arc::new(Mutex::new(LedSequence::default()));

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

                let mut video_info = VideoInfoRaw::new();

                match video_info.parse(param) {
                    Ok(info) => info,
                    Err(e) => {
                        error!("ParamChanged: Failed to parse AudioInfoRaw: {:?}", e);
                        return;
                    }
                };

                // TODO
                info!("Format negotiated: ",);
            }
        };

        let process_cb = {
            let colors_tx = colors_tx.clone();

            move |stream: &StreamRef, _data: &mut _| {
                if let Some(mut buf) = stream.dequeue_buffer() {
                    let data = &mut buf.datas_mut()[0];
                    // Data to img
                    let img = RgbImage::default();
                    let mut current = current_value.lock().unwrap();
                    parse_image(&img, &mut current);

                    let _ = colors_tx.send(current.clone());
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

        info!("PipeWire video capture started");

        mainloop.run();

        Ok(())
    }
}
