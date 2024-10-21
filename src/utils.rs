// use image::{ImageBuffer, Rgba};
// use rgb::RGBA8;

// pub fn avg_color(img: ImageBuffer<Rgba<u8>, Vec<u8>>) -> RGBA8 {
//     let (width, height) = img.dimensions();
//     let total_pixels = (width * height) as u64;

//     let mut total_r: u64 = 0;
//     let mut total_g: u64 = 0;
//     let mut total_b: u64 = 0;
//     let mut total_a: u64 = 0;

//     for pixel in img.pixels() {
//         let [r, g, b, a] = pixel.0;
//         total_r += r as u64;
//         total_g += g as u64;
//         total_b += b as u64;
//         total_a += a as u64;
//     }

//     let avg_r = (total_r / total_pixels) as u8;
//     let avg_g = (total_g / total_pixels) as u8;
//     let avg_b = (total_b / total_pixels) as u8;
//     let avg_a = (total_a / total_pixels) as u8;

//     RGBA8::new(avg_r, avg_g, avg_b, avg_a)
// }

use rgb::RGB8;

pub fn hex_to_rgb(hex: &str) -> RGB8 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    RGB8::new(r, g, b)
}
