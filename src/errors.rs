use derive_more::{Display, From};
use image::ImageError;
use std::{env, io, num, str};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    // -- Decoding
    #[from]
    Utf8Error(str::Utf8Error),
    #[from]
    ParseIntError(num::ParseIntError),
    #[from]
    ImageError(ImageError),

    // -- IO
    #[from]
    VarError(env::VarError),
    #[from]
    Config(confique::Error),
    #[from]
    SerialPort(serialport::Error),
    #[from]
    XCapError(xcap::XCapError),
    #[from]
    PipewireError(pipewire::Error),

    #[from]
    #[display("given {given}")]
    WrongWallpaperPath { given: String },

    // -- Strip
    #[from]
    PostfixReading(io::Error),
    #[from]
    #[display("given {given} must be {actual}")]
    WrongLength { given: usize, actual: usize },
    #[from]
    #[display("{_0:?}")]
    WrongPostfix([u8; 3]),
}
