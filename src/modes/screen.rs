use confique::Config;
use image::{ImageBuffer, RgbImage};
use pipewire::context::Context;
use pipewire::keys;
use pipewire::main_loop::MainLoop;
use pipewire::properties::properties;
use pipewire::spa::param::format::{MediaSubtype, MediaType};
use pipewire::spa::param::format_utils;
use pipewire::spa::param::video::{VideoFormat, VideoInfoRaw};
use pipewire::spa::param::ParamType;
use pipewire::spa::utils::Direction;
use pipewire::stream::{Stream, StreamFlags};
use std::sync::{Arc, Mutex};

use crate::config::CONFIG;
use crate::errors::Result;
use crate::modes::Mode;
use crate::strip::Strip;

#[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
pub struct ScreenModConf {
    #[config(default = 30)]
    pub capture_fps: u32,
}

#[derive(Clone, Debug)]
struct ScreenProcessData {
    pub image_buffer: Arc<Mutex<Option<RgbImage>>>,
    pub width: Arc<Mutex<u32>>,
    pub height: Arc<Mutex<u32>>,
}

impl Mode {
    pub async fn poll_screen(&self, strip: Arc<Strip>) -> Result<()> {
        let image_buffer = Arc::new(Mutex::new(None));
        let width = Arc::new(Mutex::new(0));
        let height = Arc::new(Mutex::new(0));

        let data = ScreenProcessData {
            image_buffer: image_buffer.clone(),
            width: width.clone(),
            height: height.clone(),
        };

        let mainloop = MainLoop::new(None)?;
        let context = Context::new(&mainloop)?;
        let core = context.connect(None)?;

        let props = properties! {
            *keys::APP_NAME => "PLight Screen",
            *keys::MEDIA_TYPE => "Video",
            *keys::MEDIA_CATEGORY => "Capture",
            *keys::MEDIA_ROLE => "Screen",
            *keys::NODE_NAME => "screen-capture",
            // *keys::STREAM_CAPTURE_SCREEN => "true",
            // *keys::STREAM_DONT_RECONNECT => "true",
            // *keys::WINDOW_WIDTH => "1920",
            // *keys::WINDOW_HEIGHT => "1080",
        };

        let stream = Stream::new(&core, "Screen Capture", props)?;

        stream.connect(
            Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut [],
        )?;

        let _listener = stream
            .add_local_listener_with_user_data(data)
            .param_changed(|_, _, id, param| {
                if id != ParamType::Format.as_raw() {
                    return;
                }

                // let (media_type, media_subtype) = match format_utils::parse_format(param) {
                //     Ok(v) => v,
                //     Err(_) => return,
                // };

                // if media_type != MediaType::Video || media_subtype != MediaSubtype::Raw {
                //     return;
                // }

                // if let Ok((_, format)) = VideoFormat::parse(param) {
                //     if let Some(video_info) = VideoInfo::from_format(param, format) {
                //         let mut width = video_info.size().0;
                //         let mut height = video_info.size().1;

                //         // Handle stride if necessary
                //         if video_info.stride() > 0 {
                //             width = video_info.stride() as u32 / 3; // For RGB24
                //         }

                //         *data.width.lock().unwrap() = width;
                //         *data.height.lock().unwrap() = height;
                //     }
                // }
            })
            .process(|stream, spd| {
                if let Some(mut buf) = stream.dequeue_buffer() {
                    let datas = buf.datas_mut();
                    if datas.is_empty() {
                        return;
                    }

                    let width = *spd.width.lock().unwrap();
                    let height = *spd.height.lock().unwrap();

                    if width == 0 || height == 0 {
                        return;
                    }

                    // println!("Stream");

                    // *spd.image_buffer.lock().unwrap() = Some(img);
                }
                stream.flush(false).unwrap();
            })
            .register()?;

        mainloop.run();
        Ok(())
    }
}

// use confique::Config;
// use serde::{Deserialize, Serialize};
// use std::str;
// use xcap::Monitor;

// use crate::config::CONFIG;
// use crate::errors::Result;
// use crate::modes::Mode;
// use crate::strip::Strip;
// use crate::utils::{parse_image, rgba8_to_rgb8};

// #[derive(Clone, PartialEq, PartialOrd, Debug, Config)]
// pub struct ScreenModConf {
//     #[config(default = "XCap")]
//     pub engine: CaptureEngine,
// }

// impl Mode {
//     pub async fn poll_screen(&self, strip: &mut Strip) -> Result<()> {
//         let monitor = Monitor::all()?[0].clone();

//         loop {
//             let image = match CONFIG.modes.screen.engine {
//                 CaptureEngine::XCap => monitor.capture_image()?,
//             };

//             // TODO: Maybe there is better way to convert buffer to buffer without alpha
//             strip.set_leds(&parse_image(&rgba8_to_rgb8(image)).await)?;
//         }
//     }
// }

// #[derive(Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
// pub enum CaptureEngine {
//     XCap,
// }
