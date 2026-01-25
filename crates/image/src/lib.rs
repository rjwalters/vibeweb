//! Image decoding (PNG, JPEG)
//!
//! This crate provides image decoding functionality for the Vibeweb browser.
//! Currently supports PNG images, with JPEG support planned for the future.
//!
//! # Example
//!
//! ```no_run
//! use vw_image::{decode, decode_png, PixelFormat};
//!
//! // Decode PNG from bytes
//! let png_bytes = std::fs::read("image.png").unwrap();
//! let image = decode_png(&png_bytes).unwrap();
//!
//! println!("Image: {}x{}", image.width, image.height);
//! println!("Format: {:?}", image.format);
//! ```

mod decode;
mod error;

pub use decode::{decode, decode_png, DecodedImage, PixelFormat};
pub use error::ImageError;
