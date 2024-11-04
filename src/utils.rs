use image::{ImageBuffer, Pixel, Rgb};
use ndarray::{s, Array2};

use crate::config::StripConf;

pub fn parse_image<F>(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    mut process: F,
    strip_config: &StripConf,
) -> Vec<Rgb<u8>>
where
    F: FnMut(&[Rgb<u8>]) -> Rgb<u8>,
{
    let dim = img.dimensions();
    let (width_p, height_p) = (dim.0 as usize, dim.1 as usize);

    let mut colors = Vec::<Rgb<u8>>::with_capacity(
        (2 * (strip_config.width + strip_config.height) - strip_config.bottom_gap).into(),
    );

    let horizontal_thickness_p = (width_p - strip_config.corner_size_p * 2) / strip_config.width;
    let vertical_thickness_p = (height_p - strip_config.corner_size_p * 2) / strip_config.height;

    // ? Maybe its better to use Array2<&Rgb<u8>>
    let pixels: Array2<Rgb<u8>> = Array2::from_shape_fn((height_p, width_p), |(y, x)| {
        *img.get_pixel(x as u32, y as u32)
    });

    let half_bottom_length = (strip_config.width - strip_config.bottom_gap) / 2;

    // Bottom right
    let right_bottom_offset_p =
        strip_config.corner_size_p + (half_bottom_length * horizontal_thickness_p);
    for i in (0..half_bottom_length).rev() {
        let slice = pixels
            .slice(s![
                (height_p - strip_config.thickness_p)..height_p,
                (right_bottom_offset_p + (i * horizontal_thickness_p))
                    ..(right_bottom_offset_p + ((i + 1) * horizontal_thickness_p))
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Right
    for i in (0..strip_config.height).rev() {
        let slice = pixels
            .slice(s![
                (i * vertical_thickness_p)..((i + 1) * vertical_thickness_p),
                (width_p - strip_config.thickness_p)..width_p
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Top
    for i in (0..strip_config.width).rev() {
        let slice = pixels
            .slice(s![
                0..strip_config.thickness_p,
                (i * horizontal_thickness_p)..((i + 1) * horizontal_thickness_p)
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Left
    for i in 0..strip_config.height {
        let slice = pixels
            .slice(s![
                (i * vertical_thickness_p)..((i + 1) * vertical_thickness_p),
                0..strip_config.thickness_p,
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    // Bottom left
    for i in 0..half_bottom_length {
        let slice = pixels
            .slice(s![
                (height_p - strip_config.thickness_p)..height_p,
                (i * horizontal_thickness_p)..((i + 1) * horizontal_thickness_p)
            ])
            .to_owned();
        let flat = slice.to_shape(slice.len()).unwrap();

        colors.push(process(flat.as_slice().unwrap()));
    }

    colors
}

pub fn average_color(pixels: &[Rgb<u8>]) -> Rgb<u8> {
    let total_pixels = pixels.len() as u64;

    let mut total_r: u64 = 0;
    let mut total_g: u64 = 0;
    let mut total_b: u64 = 0;

    for pixel in pixels {
        let [r, g, b] = pixel.0;
        total_r += r as u64;
        total_g += g as u64;
        total_b += b as u64;
    }

    let avg_r = (total_r / total_pixels) as u8;
    let avg_g = (total_g / total_pixels) as u8;
    let avg_b = (total_b / total_pixels) as u8;

    Rgb::<u8>::from([avg_r, avg_g, avg_b])
}

pub fn hex_to_rgb(hex: &str) -> Rgb<u8> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    Rgb::<u8>::from([r, g, b])
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
