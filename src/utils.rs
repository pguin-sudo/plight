pub mod math;
mod parse_modes;
pub mod time;

use image::{ImageBuffer, Pixel, Rgb, Rgba};
use ndarray::{s, Array2};
use parse_modes::{average, median};

use crate::{config::CONFIG, errors::Result};

pub async fn parse_image(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<Rgb<u8>> {
    let process = match CONFIG.global.parse_mode {
        crate::config::ParseMode::Average => average,
        crate::config::ParseMode::Median => median,
    };

    let dim = img.dimensions();
    let (width_p, height_p) = (dim.0 as usize, dim.1 as usize);

    let mut colors = Vec::<Rgb<u8>>::with_capacity(
        (2 * (CONFIG.strip.width + CONFIG.strip.height) - CONFIG.strip.bottom_gap).into(),
    );

    let horizontal_thickness_p = (width_p - CONFIG.strip.corner_size_p * 2) / CONFIG.strip.width;
    let vertical_thickness_p = (height_p - CONFIG.strip.corner_size_p * 2) / CONFIG.strip.height;

    // ? Maybe its better to use Array2<&Rgb<u8>>
    let pixels: Array2<Rgb<u8>> = Array2::from_shape_fn((height_p, width_p), |(y, x)| {
        *img.get_pixel(x as u32, y as u32)
    });

    let half_bottom_length = (CONFIG.strip.width - CONFIG.strip.bottom_gap) / 2;

    // Bottom right
    let right_bottom_offset_p =
        CONFIG.strip.corner_size_p + (half_bottom_length * horizontal_thickness_p);
    for i in (0..half_bottom_length).rev() {
        let slice = pixels
            .slice(s![
                (height_p - CONFIG.strip.thickness_p)..height_p,
                (right_bottom_offset_p + (i * horizontal_thickness_p))
                    ..(right_bottom_offset_p + ((i + 1) * horizontal_thickness_p))
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Right
    for i in (0..CONFIG.strip.height).rev() {
        let slice = pixels
            .slice(s![
                (i * vertical_thickness_p)..((i + 1) * vertical_thickness_p),
                (width_p - CONFIG.strip.thickness_p)..width_p
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Top
    for i in (0..CONFIG.strip.width).rev() {
        let slice = pixels
            .slice(s![
                0..CONFIG.strip.thickness_p,
                (i * horizontal_thickness_p)..((i + 1) * horizontal_thickness_p)
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Left
    for i in 0..CONFIG.strip.height {
        let slice = pixels
            .slice(s![
                (i * vertical_thickness_p)..((i + 1) * vertical_thickness_p),
                0..CONFIG.strip.thickness_p,
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Bottom left
    for i in 0..half_bottom_length {
        let slice = pixels
            .slice(s![
                (height_p - CONFIG.strip.thickness_p)..height_p,
                (i * horizontal_thickness_p)..((i + 1) * horizontal_thickness_p)
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    colors
}

pub fn hex_to_rgb(hex: &str) -> Result<Rgb<u8>> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    Ok(Rgb::<u8>::from([r, g, b]))
}

pub fn rgba8_to_rgb8(
    input: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let width = input.width() as usize;
    let height = input.height() as usize;

    let input: &Vec<u8> = input.as_raw();

    let mut output_data = vec![0u8; width * height * 3];

    let mut i = 0;
    for chunk in input.chunks(4) {
        output_data[i..i + 3].copy_from_slice(&chunk[0..3]);
        i += 3;
    }

    image::ImageBuffer::from_raw(width as u32, height as u32, output_data).unwrap()
}

// #[deprecated]
pub fn rotate_smooth(colors: &[Rgb<u8>], speed: f32) -> Vec<Rgb<u8>> {
    let mut result = Vec::<Rgb<u8>>::with_capacity(colors.len());
    for i in 0..(colors.len() - 1) {
        result.push(colors[i].map2(&colors[i + 1], |channel1, chanel2| {
            (channel1 as f32 * speed).round() as u8 + (chanel2 as f32 * (1.0 - speed)).round() as u8
        }))
    }
    result.push(
        colors[colors.len() - 1].map2(&colors[0], |channel1, chanel2| {
            (channel1 as f32 * speed).round() as u8 + (chanel2 as f32 * (1.0 - speed)).round() as u8
        }),
    );

    result
}
