//! Image decoding functionality

use crate::error::ImageError;
use std::io::Cursor;

/// Supported pixel formats for decoded images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// Red, Green, Blue (3 bytes per pixel)
    Rgb,
    /// Red, Green, Blue, Alpha (4 bytes per pixel)
    Rgba,
    /// Grayscale (1 byte per pixel)
    Gray,
    /// Grayscale with alpha (2 bytes per pixel)
    GrayAlpha,
}

impl PixelFormat {
    /// Returns the number of bytes per pixel for this format
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Rgb => 3,
            PixelFormat::Rgba => 4,
            PixelFormat::Gray => 1,
            PixelFormat::GrayAlpha => 2,
        }
    }
}

/// A decoded image with pixel data
#[derive(Debug, Clone)]
pub struct DecodedImage {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Raw pixel data (row-major, top-to-bottom)
    pub data: Vec<u8>,
}

impl DecodedImage {
    /// Create a new decoded image
    pub fn new(width: u32, height: u32, format: PixelFormat, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            format,
            data,
        }
    }

    /// Returns the expected size in bytes for the pixel data
    pub fn expected_data_size(&self) -> usize {
        (self.width as usize) * (self.height as usize) * self.format.bytes_per_pixel()
    }

    /// Returns true if the pixel data has the expected size
    pub fn has_valid_data_size(&self) -> bool {
        self.data.len() == self.expected_data_size()
    }

    /// Get a pixel at the given coordinates
    ///
    /// Returns None if coordinates are out of bounds.
    /// Returns pixel data as a slice of bytes in the image's format.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let bpp = self.format.bytes_per_pixel();
        let offset = ((y as usize) * (self.width as usize) + (x as usize)) * bpp;
        Some(&self.data[offset..offset + bpp])
    }

    /// Convert the image to RGBA format
    ///
    /// If the image is already RGBA, returns a clone.
    /// Otherwise, converts the pixel data to RGBA format.
    pub fn to_rgba(&self) -> DecodedImage {
        if self.format == PixelFormat::Rgba {
            return self.clone();
        }

        let num_pixels = (self.width as usize) * (self.height as usize);
        let mut rgba_data = Vec::with_capacity(num_pixels * 4);

        match self.format {
            PixelFormat::Rgba => unreachable!(),
            PixelFormat::Rgb => {
                for chunk in self.data.chunks_exact(3) {
                    rgba_data.push(chunk[0]); // R
                    rgba_data.push(chunk[1]); // G
                    rgba_data.push(chunk[2]); // B
                    rgba_data.push(255); // A (fully opaque)
                }
            }
            PixelFormat::Gray => {
                for &gray in &self.data {
                    rgba_data.push(gray); // R
                    rgba_data.push(gray); // G
                    rgba_data.push(gray); // B
                    rgba_data.push(255); // A (fully opaque)
                }
            }
            PixelFormat::GrayAlpha => {
                for chunk in self.data.chunks_exact(2) {
                    let gray = chunk[0];
                    let alpha = chunk[1];
                    rgba_data.push(gray); // R
                    rgba_data.push(gray); // G
                    rgba_data.push(gray); // B
                    rgba_data.push(alpha); // A
                }
            }
        }

        DecodedImage::new(self.width, self.height, PixelFormat::Rgba, rgba_data)
    }
}

/// PNG file signature (magic bytes)
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Detect if data is a PNG image
fn is_png(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == PNG_SIGNATURE
}

/// Decode an image from bytes, auto-detecting the format
///
/// Currently supports:
/// - PNG
///
/// Future support planned for:
/// - JPEG
///
/// # Errors
///
/// Returns `ImageError::UnsupportedFormat` if the format cannot be detected.
/// Returns other `ImageError` variants for decoding failures.
pub fn decode(data: &[u8]) -> Result<DecodedImage, ImageError> {
    if is_png(data) {
        decode_png(data)
    } else {
        Err(ImageError::UnsupportedFormat)
    }
}

/// Decode a PNG image from bytes
///
/// # Errors
///
/// Returns `ImageError::InvalidData` if the PNG is malformed.
/// Returns `ImageError::Io` for I/O errors during decoding.
pub fn decode_png(data: &[u8]) -> Result<DecodedImage, ImageError> {
    let cursor = Cursor::new(data);
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;

    // Truncate buffer to actual size
    buf.truncate(info.buffer_size());

    let format = match info.color_type {
        png::ColorType::Rgb => PixelFormat::Rgb,
        png::ColorType::Rgba => PixelFormat::Rgba,
        png::ColorType::Grayscale => PixelFormat::Gray,
        png::ColorType::GrayscaleAlpha => PixelFormat::GrayAlpha,
        png::ColorType::Indexed => {
            // For indexed color, the png crate expands to RGB/RGBA
            // depending on whether there's transparency
            return Err(ImageError::InvalidData(
                "indexed PNG not expanded by decoder".to_string(),
            ));
        }
    };

    Ok(DecodedImage::new(info.width, info.height, format, buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid 1x1 red PNG (created programmatically)
    fn create_test_png() -> Vec<u8> {
        // This is a minimal 1x1 red RGBA PNG
        // Generated using the png crate itself
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            // Red pixel: R=255, G=0, B=0, A=255
            writer.write_image_data(&[255, 0, 0, 255]).unwrap();
        }
        buf
    }

    fn create_grayscale_png() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, 2, 2);
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            // 2x2 grayscale: black, dark gray, light gray, white
            writer.write_image_data(&[0, 85, 170, 255]).unwrap();
        }
        buf
    }

    fn create_rgb_png() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, 2, 1);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            // 2x1 RGB: red, green
            writer.write_image_data(&[255, 0, 0, 0, 255, 0]).unwrap();
        }
        buf
    }

    #[test]
    fn pixel_format_bytes_per_pixel() {
        assert_eq!(PixelFormat::Rgb.bytes_per_pixel(), 3);
        assert_eq!(PixelFormat::Rgba.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::Gray.bytes_per_pixel(), 1);
        assert_eq!(PixelFormat::GrayAlpha.bytes_per_pixel(), 2);
    }

    #[test]
    fn decode_png_rgba() {
        let png_data = create_test_png();
        let image = decode_png(&png_data).unwrap();

        assert_eq!(image.width, 1);
        assert_eq!(image.height, 1);
        assert_eq!(image.format, PixelFormat::Rgba);
        assert!(image.has_valid_data_size());
        assert_eq!(image.data, vec![255, 0, 0, 255]); // Red pixel
    }

    #[test]
    fn decode_png_rgb() {
        let png_data = create_rgb_png();
        let image = decode_png(&png_data).unwrap();

        assert_eq!(image.width, 2);
        assert_eq!(image.height, 1);
        assert_eq!(image.format, PixelFormat::Rgb);
        assert!(image.has_valid_data_size());
        // Red pixel, Green pixel
        assert_eq!(image.data, vec![255, 0, 0, 0, 255, 0]);
    }

    #[test]
    fn decode_png_grayscale() {
        let png_data = create_grayscale_png();
        let image = decode_png(&png_data).unwrap();

        assert_eq!(image.width, 2);
        assert_eq!(image.height, 2);
        assert_eq!(image.format, PixelFormat::Gray);
        assert!(image.has_valid_data_size());
        assert_eq!(image.data, vec![0, 85, 170, 255]);
    }

    #[test]
    fn auto_detect_png() {
        let png_data = create_test_png();
        let image = decode(&png_data).unwrap();
        assert_eq!(image.format, PixelFormat::Rgba);
    }

    #[test]
    fn unsupported_format() {
        // Random bytes that don't match any known format
        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        let result = decode(&unknown_data);
        assert!(matches!(result, Err(ImageError::UnsupportedFormat)));
    }

    #[test]
    fn invalid_png_data() {
        // PNG signature but corrupt data
        let invalid_png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0xFF];
        let result = decode_png(&invalid_png);
        // Should return some error (could be InvalidData or Io depending on how it fails)
        assert!(result.is_err());
    }

    #[test]
    fn get_pixel() {
        let image = DecodedImage::new(
            2,
            2,
            PixelFormat::Rgba,
            vec![
                255, 0, 0, 255, // (0,0) red
                0, 255, 0, 255, // (1,0) green
                0, 0, 255, 255, // (0,1) blue
                255, 255, 0, 255, // (1,1) yellow
            ],
        );

        assert_eq!(image.get_pixel(0, 0), Some(&[255, 0, 0, 255][..]));
        assert_eq!(image.get_pixel(1, 0), Some(&[0, 255, 0, 255][..]));
        assert_eq!(image.get_pixel(0, 1), Some(&[0, 0, 255, 255][..]));
        assert_eq!(image.get_pixel(1, 1), Some(&[255, 255, 0, 255][..]));
        assert_eq!(image.get_pixel(2, 0), None); // out of bounds
        assert_eq!(image.get_pixel(0, 2), None); // out of bounds
    }

    #[test]
    fn to_rgba_from_rgb() {
        let rgb_image = DecodedImage::new(
            2,
            1,
            PixelFormat::Rgb,
            vec![
                255, 0, 0, // red
                0, 255, 0, // green
            ],
        );

        let rgba_image = rgb_image.to_rgba();
        assert_eq!(rgba_image.format, PixelFormat::Rgba);
        assert_eq!(rgba_image.width, 2);
        assert_eq!(rgba_image.height, 1);
        assert_eq!(
            rgba_image.data,
            vec![255, 0, 0, 255, 0, 255, 0, 255] // red with alpha, green with alpha
        );
    }

    #[test]
    fn to_rgba_from_gray() {
        let gray_image = DecodedImage::new(2, 1, PixelFormat::Gray, vec![0, 255]);

        let rgba_image = gray_image.to_rgba();
        assert_eq!(rgba_image.format, PixelFormat::Rgba);
        assert_eq!(
            rgba_image.data,
            vec![
                0, 0, 0, 255, // black
                255, 255, 255, 255 // white
            ]
        );
    }

    #[test]
    fn to_rgba_from_gray_alpha() {
        let gray_alpha_image =
            DecodedImage::new(2, 1, PixelFormat::GrayAlpha, vec![128, 255, 64, 128]);

        let rgba_image = gray_alpha_image.to_rgba();
        assert_eq!(rgba_image.format, PixelFormat::Rgba);
        assert_eq!(
            rgba_image.data,
            vec![
                128, 128, 128, 255, // 50% gray, opaque
                64, 64, 64, 128 // 25% gray, 50% transparent
            ]
        );
    }

    #[test]
    fn to_rgba_already_rgba() {
        let rgba_image = DecodedImage::new(1, 1, PixelFormat::Rgba, vec![100, 150, 200, 128]);

        let converted = rgba_image.to_rgba();
        assert_eq!(converted.data, rgba_image.data);
    }

    #[test]
    fn is_png_detection() {
        // Valid PNG signature
        assert!(is_png(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]));
        // Invalid - too short
        assert!(!is_png(&[0x89, 0x50, 0x4E, 0x47]));
        // Invalid - wrong signature
        assert!(!is_png(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46])); // JPEG
    }
}
