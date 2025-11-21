use std::time::Instant;

use image::{ImageBuffer, Rgb};
use log::trace;
use ndarray::{s, Array2};

use crate::config::CONFIG;
use crate::core::led_sequence::LedSequence;
use crate::utils::color_math::{average, median};

// TODO: Replace Vec with &[Rgb<u8>]>
pub fn parse_image(img: &ImageBuffer<Rgb<u8>, Vec<u8>>, led_sequence: &mut LedSequence) {
    let __debug_time = Instant::now();

    let process = match CONFIG.global.parse_mode {
        crate::config::ParseMode::Average => average,
        crate::config::ParseMode::Median => median,
    };

    let (width_p, height_p) = (img.width() as usize, img.height() as usize);
    let horizontal_thickness_p = (width_p - CONFIG.strip.corner_size_p * 2) / CONFIG.strip.width;
    let vertical_thickness_p = (height_p - CONFIG.strip.corner_size_p * 2) / CONFIG.strip.height;
    let half_bottom_length = (CONFIG.strip.width - CONFIG.strip.bottom_gap) / 2;

    let pixels: Array2<Rgb<u8>> = Array2::from_shape_fn((height_p, width_p), |(y, x)| {
        *img.get_pixel(x as u32, y as u32)
    });

    let mut colors = Vec::with_capacity(CONFIG.strip.len());

    // Bottom right
    let right_bottom_offset_p =
        CONFIG.strip.corner_size_p + (half_bottom_length * horizontal_thickness_p);
    for i in (0..half_bottom_length).rev() {
        let slice = pixels.slice(s![
            (height_p - CONFIG.strip.thickness_p)..height_p,
            (right_bottom_offset_p + (i * horizontal_thickness_p))
                ..(right_bottom_offset_p + ((i + 1) * horizontal_thickness_p))
        ]);
        colors.push(process(&slice.iter().copied().collect::<Vec<_>>()));
    }

    // Right
    for i in (0..CONFIG.strip.height).rev() {
        let slice = pixels.slice(s![
            (i * vertical_thickness_p)..((i + 1) * vertical_thickness_p),
            (width_p - CONFIG.strip.thickness_p)..width_p
        ]);
        colors.push(process(&slice.iter().copied().collect::<Vec<_>>()));
    }

    // Top
    for i in (0..CONFIG.strip.width).rev() {
        let slice = pixels.slice(s![
            0..CONFIG.strip.thickness_p,
            (i * horizontal_thickness_p)..((i + 1) * horizontal_thickness_p)
        ]);
        colors.push(process(&slice.iter().copied().collect::<Vec<_>>()));
    }

    // Left
    for i in 0..CONFIG.strip.height {
        let slice = pixels.slice(s![
            (i * vertical_thickness_p)..((i + 1) * vertical_thickness_p),
            0..CONFIG.strip.thickness_p,
        ]);
        colors.push(process(&slice.iter().copied().collect::<Vec<_>>()));
    }

    // Bottom left
    for i in 0..half_bottom_length {
        let slice = pixels.slice(s![
            (height_p - CONFIG.strip.thickness_p)..height_p,
            (i * horizontal_thickness_p)..((i + 1) * horizontal_thickness_p)
        ]);
        colors.push(process(&slice.iter().copied().collect::<Vec<_>>()));
    }

    led_sequence.set_colors(&colors);
    trace!("Image processing duration: {:?}", __debug_time.elapsed());
}
