//! Image decoding errors

use std::fmt;
use std::io;

/// Errors that can occur during image decoding
#[derive(Debug)]
pub enum ImageError {
    /// The image format is not supported
    UnsupportedFormat,

    /// The image data is invalid or corrupt
    InvalidData(String),

    /// An I/O error occurred while reading the image
    Io(io::Error),
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageError::UnsupportedFormat => write!(f, "unsupported image format"),
            ImageError::InvalidData(msg) => write!(f, "invalid image data: {}", msg),
            ImageError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ImageError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ImageError {
    fn from(err: io::Error) -> Self {
        ImageError::Io(err)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = ImageError::UnsupportedFormat;
        assert_eq!(format!("{}", err), "unsupported image format");

        let err = ImageError::InvalidData("test error".to_string());
        assert_eq!(format!("{}", err), "invalid image data: test error");
    }

    #[test]
    fn error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let img_err: ImageError = io_err.into();
        assert!(matches!(img_err, ImageError::Io(_)));
    }
}
