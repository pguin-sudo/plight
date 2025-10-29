use image::{Pixel, Rgb};

pub fn average(pixels: &[Rgb<u8>]) -> Rgb<u8> {
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

pub fn median(pixels: &[Rgb<u8>]) -> Rgb<u8> {
    let mut r_values: Vec<u8> = pixels.iter().map(|pixel| pixel.0[0]).collect();
    let mut g_values: Vec<u8> = pixels.iter().map(|pixel| pixel.0[1]).collect();
    let mut b_values: Vec<u8> = pixels.iter().map(|pixel| pixel.0[2]).collect();

    r_values.sort();
    g_values.sort();
    b_values.sort();

    let mid_index = pixels.len() / 2;

    let median_r = r_values[mid_index];
    let median_g = g_values[mid_index];
    let median_b = b_values[mid_index];

    Rgb::<u8>::from([median_r, median_g, median_b])
}

#[deprecated]
pub fn rotate_smooth(colors: &[Rgb<u8>], speed: f32) -> Vec<Rgb<u8>> {
    let mut result = Vec::with_capacity(colors.len());
    for i in 0..colors.len() - 1 {
        result.push(colors[i].map2(&colors[i + 1], |channel1, channel2| {
            (channel1 as f32 * speed).round() as u8
                + (channel2 as f32 * (1.0 - speed)).round() as u8
        }));
    }
    result.push(
        colors[colors.len() - 1].map2(&colors[0], |channel1, channel2| {
            (channel1 as f32 * speed).round() as u8
                + (channel2 as f32 * (1.0 - speed)).round() as u8
        }),
    );

    result
}
