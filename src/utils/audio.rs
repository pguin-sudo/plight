use crate::config::CONFIG;
use anyhow::{bail, Context, Result};
use log::trace;
use pipewire::spa::{
    buffer::Data,
    param::audio::{AudioFormat, AudioInfoRaw},
};
use std::convert::TryInto;
use thiserror::Error;
use unit_interval::UnitInterval;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No data in buffer")]
    NoData,
    #[error("Unsupported channel count: {0} (must be 1 or 2)")]
    UnsupportedChannels(usize),
    #[error("Unsupported audio format: {0:?}")]
    UnsupportedFormat(AudioFormat),
    #[error("Buffer too short for a single frame")]
    BufferTooShort,
    #[error("Invalid sample bytes length: {0}")]
    InvalidBytes(usize),
}

pub fn make_get_sound_level_closure(
    audio_info: AudioInfoRaw,
) -> impl FnMut(&mut Data) -> Result<(UnitInterval<f64>, UnitInterval<f64>)> {
    trace!("Audio format: {:?}", audio_info.format());
    move |data: &mut Data| {
        let audio_data = data.data().context(AudioError::NoData)?;
        if audio_data.is_empty() {
            return Ok((UnitInterval::zero(), UnitInterval::zero()));
        }
        let channels = validate_channels(audio_info.channels() as usize)?;
        let (sample_size, is_be) = get_sample_size_and_endianness(audio_info.format())?;
        let stride = sample_size * channels;
        if audio_data.len() < stride {
            return Ok((UnitInterval::zero(), UnitInterval::zero()));
        }
        let num_frames = audio_data.len() / stride;
        trace!(
            "Channels: {}, Sample size: {}, Num frames: {}",
            channels,
            sample_size,
            num_frames
        );
        let (left_sum_sq, right_sum_sq) = compute_sum_of_squares(
            audio_data,
            channels,
            sample_size,
            stride,
            is_be,
            &audio_info,
        )?;
        let left_rms = (left_sum_sq / num_frames as f64).sqrt();
        let right_rms = (right_sum_sq / num_frames as f64).sqrt();
        trace!("RMS: left = {}, right = {}", left_rms, right_rms);
        let db_range = CONFIG.behavior.audio.max_level_db - CONFIG.behavior.audio.min_level_db;
        let min_rms = 10.0f64.powf(CONFIG.behavior.audio.min_level_db / 20.0);
        trace!("Min RMS threshold: {}", min_rms);
        let convert_to_level = |rms: f64| -> UnitInterval<f64> {
            if rms <= min_rms || (rms < 0.005 && right_rms == 0.0) {
                UnitInterval::zero()
            } else {
                let db = 20.0 * rms.log10();
                let normalized = (db - CONFIG.behavior.audio.min_level_db) / db_range;
                UnitInterval::new_clamped(normalized)
            }
        };
        let left_level = convert_to_level(left_rms);
        let right_level = convert_to_level(right_rms);
        trace!(
            "Levels: left = {}, right = {}",
            *left_level.as_inner(),
            *right_level.as_inner()
        );
        Ok((left_level, right_level))
    }
}

fn validate_channels(channels: usize) -> Result<usize> {
    if channels == 0 || channels > 2 {
        bail!(AudioError::UnsupportedChannels(channels));
    }
    Ok(channels)
}

fn get_sample_size_and_endianness(format: AudioFormat) -> Result<(usize, bool)> {
    match format {
        AudioFormat::S16LE => Ok((2, false)),
        AudioFormat::S16BE => Ok((2, true)),
        AudioFormat::S32LE => Ok((4, false)),
        AudioFormat::S32BE => Ok((4, true)),
        AudioFormat::F32LE => Ok((4, false)),
        AudioFormat::F32BE => Ok((4, true)),
        AudioFormat::F64LE => Ok((8, false)),
        AudioFormat::F64BE => Ok((8, true)),
        fmt => bail!(AudioError::UnsupportedFormat(fmt)),
    }
}

fn compute_sum_of_squares(
    audio_data: &[u8],
    channels: usize,
    sample_size: usize,
    stride: usize,
    is_be: bool,
    audio_info: &AudioInfoRaw,
) -> Result<(f64, f64)> {
    let mut left_sum_sq: f64 = 0.0;
    let mut right_sum_sq: f64 = 0.0;
    let mut left_max_abs: f64 = 0.0;
    let mut right_max_abs: f64 = 0.0;
    for frame in audio_data.chunks_exact(stride) {
        let left = sample_to_f64(&frame[0..sample_size], sample_size, is_be, audio_info)
            .with_context(|| AudioError::InvalidBytes(frame.len()))?;
        left_sum_sq += left * left;
        left_max_abs = left_max_abs.max(left.abs());
        let right = if channels == 2 {
            sample_to_f64(&frame[sample_size..], sample_size, is_be, audio_info)
                .with_context(|| AudioError::InvalidBytes(frame.len()))?
        } else {
            left
        };
        right_sum_sq += right * right;
        right_max_abs = right_max_abs.max(right.abs());
    }
    trace!(
        "Max abs sample: left = {}, right = {}",
        left_max_abs,
        right_max_abs
    );
    Ok((left_sum_sq, right_sum_sq))
}

fn sample_to_f64(
    bytes: &[u8],
    sample_size: usize,
    is_be: bool,
    audio_info: &AudioInfoRaw,
) -> Result<f64> {
    if bytes.len() != sample_size {
        bail!(AudioError::InvalidBytes(bytes.len()));
    }
    Ok(match sample_size {
        2 => i16_sample_to_f64(bytes, is_be)?,
        4 if matches!(audio_info.format(), AudioFormat::F32LE | AudioFormat::F32BE) => {
            f32_sample_to_f64(bytes, is_be)?
        }
        4 => i32_sample_to_f64(bytes, is_be)?,
        8 => f64_sample_to_f64(bytes, is_be)?,
        _ => bail!(AudioError::UnsupportedFormat(audio_info.format())),
    })
}

const I16_NORM: f64 = 32767.0;
const I32_NORM: f64 = 2_147_483_647.0;
const FLOAT_NOISE_THRESHOLD: f64 = 0.03;

fn i16_sample_to_f64(bytes: &[u8], is_be: bool) -> Result<f64> {
    let val = if is_be {
        i16::from_be_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    } else {
        i16::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    };
    Ok(if val.abs() <= 1 {
        0.0
    } else {
        val as f64 / I16_NORM
    })
}

fn i32_sample_to_f64(bytes: &[u8], is_be: bool) -> Result<f64> {
    let val = if is_be {
        i32::from_be_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    } else {
        i32::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    };
    Ok(if val.abs() <= 1 {
        0.0
    } else {
        val as f64 / I32_NORM
    })
}

fn f32_sample_to_f64(bytes: &[u8], is_be: bool) -> Result<f64> {
    let val = if is_be {
        f32::from_be_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    } else {
        f32::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    };
    Ok(if (val as f64).abs() <= FLOAT_NOISE_THRESHOLD {
        0.0
    } else {
        val as f64
    })
}

fn f64_sample_to_f64(bytes: &[u8], is_be: bool) -> Result<f64> {
    let val = if is_be {
        f64::from_be_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    } else {
        f64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| AudioError::InvalidBytes(bytes.len()))?,
        )
    };
    Ok(if val.abs() <= FLOAT_NOISE_THRESHOLD {
        0.0
    } else {
        val
    })
}

pub fn smooth_audio_level(target: UnitInterval<f64>, prev: UnitInterval<f64>) -> UnitInterval<f64> {
    if *target.as_inner() == 0.0 {
        return UnitInterval::new_clamped(0.0);
    }
    UnitInterval::new_clamped(if target > prev {
        prev + (target - prev) * CONFIG.behavior.audio.attack
    } else {
        prev + (target - prev) * CONFIG.behavior.audio.decay
    })
}
