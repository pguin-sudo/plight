pub fn calculate_sound_level(buffer: &[u8]) -> f64 {
    let samples = buffer
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();

    let rms =
        (samples.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / samples.len() as f64).sqrt();

    if rms <= 0.0001 {
        0.0
    } else {
        let db = 20.0 * rms.log10();
        ((db + 40.0 + 10.0) / 40.0).clamp(0.0, 1.0)
    }
}
