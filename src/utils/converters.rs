use anyhow::Result;
use image::Rgb;
use image::Rgba;

pub fn hex_to_rgb(hex: &str) -> Result<Rgb<u8>> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    Ok(Rgb([r, g, b]))
}

pub fn rgba8_to_rgb8(
    input: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = (input.width() as usize, input.height() as usize);
    let input: &[u8] = input.as_raw();
    let mut output_data = Vec::with_capacity(width * height * 3);

    for chunk in input.chunks(4) {
        output_data.extend_from_slice(&chunk[0..3]);
    }

    image::ImageBuffer::from_raw(width as u32, height as u32, output_data).unwrap()
}
