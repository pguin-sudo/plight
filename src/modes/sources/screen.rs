use std::mem;
use std::os::fd::AsRawFd;
use std::ptr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use anyhow::Result;
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use ashpd::desktop::PersistMode;
use confique::Config;
use image::{ImageBuffer, RgbImage};
use log::{debug, error, info, warn};
use pipewire::context::Context;
use pipewire::core::Core;
use pipewire::keys;
use pipewire::main_loop::MainLoop;
use pipewire::properties::properties;
use pipewire::spa::param::format::{FormatProperties, MediaSubtype, MediaType};
use pipewire::spa::param::video::{VideoFormat, VideoInfoRaw};
use pipewire::spa::param::{format_utils, ParamType};
use pipewire::spa::pod::serialize::PodSerializer;
use pipewire::spa::pod::{object, property, Pod, Value};
use pipewire::spa::utils::{Direction, Fraction, Rectangle, SpaTypes};
use pipewire::stream::{Stream, StreamFlags, StreamListener, StreamRef};
use pipewire_sys;

use crate::core::led_sequence::LedSequence;
use crate::modes::sources::Source;
use crate::utils::converters::rgba8_to_rgb8;
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
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = Self::run_pipewire_loop(colors_tx).await {
                    error!("PipeWire thread error: {}", e);
                }
            });
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
    async fn run_pipewire_loop(colors_tx: mpsc::Sender<LedSequence>) -> Result<()> {
        let proxy = Screencast::new().await?;
        let session = proxy.create_session().await?;
        proxy
            .select_sources(
                &session,
                CursorMode::Embedded,
                SourceType::Monitor | SourceType::Window,
                true,
                None, // TODO: Enable token restoration
                PersistMode::DoNot,
            )
            .await?;
        let response = proxy.start(&session, None).await?.response()?;
        let streams = response.streams();
        if streams.is_empty() {
            return Err(anyhow::anyhow!("No streams available from portal"));
        }
        let node_id = streams[0].pipe_wire_node_id();
        let fd = proxy.open_pipe_wire_remote(&session).await?;
        let mainloop = MainLoop::new(None)?;
        let context = Context::new(&mainloop)?;
        let core_ptr = unsafe {
            pipewire_sys::pw_context_connect_fd(
                context.as_raw_ptr(),
                fd.as_raw_fd(),
                ptr::null_mut(),
                0,
            )
        };

        if core_ptr.is_null() {
            return Err(anyhow::anyhow!("Failed to connect to PipeWire FD"));
        }

        let core: Core = unsafe { mem::transmute(core_ptr) };

        let props = properties! {
            *keys::MEDIA_NAME => "Screen Capture",
            *keys::MEDIA_ROLE => "Screen",
            *keys::MEDIA_CLASS => "Video/Source",
            *keys::MEDIA_CATEGORY => "Capture",
            *keys::NODE_NAME => "PLight",
            *keys::CLIENT_NAME => "PLight",
        };

        debug!("Before");
        // TODO: Fix this seg fault
        let stream = Stream::new(&core, "screen-capture", props)?;
        debug!("After");

        let obj = object!(
            SpaTypes::ObjectParamFormat,
            ParamType::EnumFormat,
            property!(FormatProperties::MediaType, Id, MediaType::Video),
            property!(FormatProperties::MediaSubtype, Id, MediaSubtype::Raw),
            property!(
                FormatProperties::VideoFormat,
                Choice,
                Enum,
                Id,
                VideoFormat::RGB,
                VideoFormat::RGBA,
                VideoFormat::RGBx,
                VideoFormat::BGR,
                VideoFormat::BGRA,
                VideoFormat::BGRx,
            ),
            property!(
                FormatProperties::VideoSize,
                Rectangle,
                Rectangle {
                    width: 1920u32,
                    height: 1080u32
                }
            ),
            property!(
                FormatProperties::VideoFramerate,
                Fraction,
                Fraction { num: 30, denom: 1 }
            ),
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
            Some(node_id),
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut params,
        )?;

        let (info_tx, info_rx) = mpsc::channel();
        let param_changed_cb =
            move |_listener: &StreamRef, _object_id: &mut _, id: u32, param: Option<&Pod>| {
                if id != ParamType::Format.as_raw() {
                    return;
                }
                let Some(pod) = param else { return };
                let Ok((media_type, media_subtype)) = format_utils::parse_format(pod) else {
                    error!("Failed to parse format POD");
                    return;
                };
                if media_type != MediaType::Video || media_subtype != MediaSubtype::Raw {
                    warn!("Unexpected format: {:?}/{:?}", media_type, media_subtype);
                    return;
                }
                let mut video_info = VideoInfoRaw::new();
                match video_info.parse(pod) {
                    Ok(info) => info,
                    Err(e) => {
                        error!("Failed to parse VideoInfoRaw: {}", e);
                        return;
                    }
                };
                let _ = info_tx.send(video_info);
                info!(
                    "Negotiated video format: {:?} {}x{} @ {}/{} fps",
                    video_info.format(),
                    video_info.size().width,
                    video_info.size().height,
                    video_info.framerate().num,
                    video_info.framerate().denom
                );
            };

        let current_colors = Arc::new(Mutex::new(LedSequence::default()));
        let process_cb = {
            let colors_tx = colors_tx.clone();
            let current_colors = current_colors.clone();
            move |stream: &StreamRef, _data: &mut _| {
                while let Some(mut buffer) = stream.dequeue_buffer() {
                    let Some(data) = buffer.datas_mut().get_mut(0) else {
                        continue;
                    };
                    let chunk = data.chunk();
                    let offset = chunk.offset() as usize;
                    let size = chunk.size() as usize;
                    if size == 0 {
                        continue;
                    }
                    let mem = match data.data() {
                        Some(data) => data,
                        None => continue,
                    };
                    let bytes = &mem[offset..offset + size];
                    let info_raw = match info_rx.try_recv() {
                        Ok(info) => info,
                        Err(_) => {
                            warn!("No video info received yet; skipping frame");
                            continue;
                        }
                    };
                    let format = info_raw.format();
                    let width = info_raw.size().width;
                    let height = info_raw.size().height;
                    let img: RgbImage = match format {
                        VideoFormat::RGBA | VideoFormat::RGBx => {
                            let rgba = ImageBuffer::from_raw(width, height, bytes.to_vec())
                                .unwrap_or_default();
                            rgba8_to_rgb8(rgba)
                        }
                        VideoFormat::BGRA | VideoFormat::BGRx => {
                            let bgra: ImageBuffer<image::Rgba<u8>, _> =
                                ImageBuffer::from_raw(width, height, bytes.to_vec())
                                    .unwrap_or_default();
                            rgba8_to_rgb8(bgra)
                        }
                        VideoFormat::RGB | VideoFormat::BGR => {
                            let rgb_data: Vec<u8> = bytes
                                .chunks_exact(3)
                                .flat_map(|p| [p[0], p[1], p[2]])
                                .collect();
                            ImageBuffer::from_raw(width, height, rgb_data).unwrap_or_default()
                        }
                        _ => {
                            warn!("Unsupported video format: {:?}", format);
                            continue;
                        }
                    };
                    if img.width() == 0 || img.height() == 0 {
                        continue;
                    }
                    let mut colors = current_colors.lock().unwrap();
                    parse_image(&img, &mut colors);
                    let _ = colors_tx.send(colors.clone());
                }
            }
        };
        let _listener: StreamListener<u32> = stream
            .add_local_listener()
            .param_changed(param_changed_cb)
            .process(process_cb)
            .register()?;
        info!("PipeWire screen capture started via portal");
        mainloop.run();
        Ok(())
    }
}
