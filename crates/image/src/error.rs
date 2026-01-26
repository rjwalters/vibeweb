//! Image decoding errors

use std::io;
use thiserror::Error;

/// Errors that can occur during image decoding
#[derive(Debug, Error)]
pub enum ImageError {
    /// The image format is not supported
    #[error("unsupported image format")]
    UnsupportedFormat,

    /// The image data is invalid or corrupt
    #[error("invalid image data: {0}")]
    InvalidData(String),

    /// An I/O error occurred while reading the image
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

impl From<png::DecodingError> for ImageError {
    fn from(err: png::DecodingError) -> Self {
        match err {
            png::DecodingError::IoError(io_err) => ImageError::Io(io_err),
            png::DecodingError::Format(msg) => {
                ImageError::InvalidData(format!("PNG format error: {}", msg))
            }
            png::DecodingError::Parameter(msg) => {
                ImageError::InvalidData(format!("PNG parameter error: {}", msg))
            }
            png::DecodingError::LimitsExceeded => {
                ImageError::InvalidData("PNG limits exceeded".to_string())
            }
        }
    }
}
