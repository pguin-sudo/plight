use thiserror::Error;
use tokio::io;

#[derive(Debug, Error)]
pub enum PLightError {
    #[error("given {given}")]
    WrongWallpaperPath { given: String },
    #[error(transparent)]
    PostfixReading(#[from] io::Error),
    #[error("given {given} must be {actual}")]
    WrongLength { given: usize, actual: usize },
    #[error("{0:?}")]
    WrongPostfix([u8; 3]),
}
