//! CPU-based framebuffer for software rendering

use crate::color::Color;
use crate::rect::Rect;
use vw_image::DecodedImage;

/// A CPU-based framebuffer for software rendering
///
/// Stores pixels in ARGB format for compatibility with softbuffer.
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Framebuffer {
    /// Create a new framebuffer with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            pixels: vec![0; size],
        }
    }

    /// Get the framebuffer width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the framebuffer height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the pixel buffer as a slice (ARGB format)
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Resize the framebuffer to new dimensions
    ///
    /// This clears the framebuffer to black.
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        let size = (width as usize) * (height as usize);
        self.pixels.resize(size, 0);
        self.pixels.fill(0);
    }

    /// Clear the entire framebuffer to a color
    pub fn clear(&mut self, color: Color) {
        let argb = color.to_argb();
        self.pixels.fill(argb);
    }

    /// Fill a rectangle with a color
    ///
    /// The rectangle is clipped to the framebuffer bounds.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        if rect.is_empty() || self.width == 0 || self.height == 0 {
            return;
        }

        // Clip rectangle to framebuffer bounds
        let x_start = rect.x.max(0) as u32;
        let y_start = rect.y.max(0) as u32;
        let x_end = (rect.right().max(0) as u32).min(self.width);
        let y_end = (rect.bottom().max(0) as u32).min(self.height);

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        let argb = color.to_argb();

        for y in y_start..y_end {
            let row_start = (y * self.width + x_start) as usize;
            let row_end = (y * self.width + x_end) as usize;
            self.pixels[row_start..row_end].fill(argb);
        }
    }

    /// Set a single pixel (bounds checked)
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.pixels[idx] = color.to_argb();
        }
    }

    /// Get a single pixel (bounds checked, returns transparent if out of bounds)
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            Color::from_argb(self.pixels[idx])
        } else {
            Color::TRANSPARENT
        }
    }

    /// Blit a decoded image to the framebuffer at the given position.
    ///
    /// The image is clipped to framebuffer bounds. Supports alpha blending
    /// for images with transparency.
    pub fn blit_image(&mut self, x: i32, y: i32, image: &DecodedImage) {
        if self.width == 0 || self.height == 0 || image.width == 0 || image.height == 0 {
            return;
        }

        // Convert to RGBA for consistent handling
        let rgba = image.to_rgba();

        for img_y in 0..rgba.height {
            let dst_y = y + img_y as i32;

            // Skip rows outside framebuffer bounds
            if dst_y < 0 || dst_y >= self.height as i32 {
                continue;
            }

            for img_x in 0..rgba.width {
                let dst_x = x + img_x as i32;

                // Skip pixels outside framebuffer bounds
                if dst_x < 0 || dst_x >= self.width as i32 {
                    continue;
                }

                if let Some(pixel) = rgba.get_pixel(img_x, img_y) {
                    let src = Color::rgba(pixel[0], pixel[1], pixel[2], pixel[3]);
                    if src.a == 255 {
                        // Opaque - direct write
                        self.set_pixel(dst_x as u32, dst_y as u32, src);
                    } else if src.a > 0 {
                        // Semi-transparent - alpha blend
                        let dst = self.get_pixel(dst_x as u32, dst_y as u32);
                        let blended = alpha_blend(src, dst);
                        self.set_pixel(dst_x as u32, dst_y as u32, blended);
                    }
                    // a == 0: fully transparent, skip
                }
            }
        }
    }
}

/// Alpha blend a source color over a destination color.
///
/// Uses the standard "over" compositing operation:
/// out = src + dst * (1 - src_alpha)
fn alpha_blend(src: Color, dst: Color) -> Color {
    let alpha = src.a as u32;
    let inv_alpha = 255 - alpha;

    let r = (src.r as u32 * alpha + dst.r as u32 * inv_alpha) / 255;
    let g = (src.g as u32 * alpha + dst.g as u32 * inv_alpha) / 255;
    let b = (src.b as u32 * alpha + dst.b as u32 * inv_alpha) / 255;

    Color::rgb(r as u8, g as u8, b as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vw_image::PixelFormat;

    #[test]
    fn framebuffer_new() {
        let fb = Framebuffer::new(100, 50);
        assert_eq!(fb.width(), 100);
        assert_eq!(fb.height(), 50);
        assert_eq!(fb.pixels().len(), 5000);
    }

    #[test]
    fn framebuffer_clear() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::RED);
        assert!(fb.pixels().iter().all(|&p| p == Color::RED.to_argb()));
    }

    #[test]
    fn framebuffer_fill_rect() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);
        fb.fill_rect(Rect::new(2, 2, 4, 4), Color::BLUE);

        // Check corner pixels
        assert_eq!(fb.get_pixel(1, 1), Color::WHITE);
        assert_eq!(fb.get_pixel(2, 2), Color::BLUE);
        assert_eq!(fb.get_pixel(5, 5), Color::BLUE);
        assert_eq!(fb.get_pixel(6, 6), Color::WHITE);
    }

    #[test]
    fn framebuffer_fill_rect_clipping() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);

        // Rectangle extends past bounds
        fb.fill_rect(Rect::new(-2, -2, 5, 5), Color::RED);

        // Check that visible part is red
        assert_eq!(fb.get_pixel(0, 0), Color::RED);
        assert_eq!(fb.get_pixel(2, 2), Color::RED);
        assert_eq!(fb.get_pixel(3, 3), Color::WHITE);
    }

    #[test]
    fn framebuffer_resize() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::RED);
        fb.resize(20, 20);
        assert_eq!(fb.width(), 20);
        assert_eq!(fb.height(), 20);
        // After resize, buffer is cleared to black
        assert_eq!(fb.get_pixel(0, 0), Color::from_argb(0));
    }

    #[test]
    fn framebuffer_set_get_pixel() {
        let mut fb = Framebuffer::new(10, 10);
        fb.set_pixel(5, 5, Color::GREEN);
        assert_eq!(fb.get_pixel(5, 5), Color::GREEN);
        assert_eq!(fb.get_pixel(0, 0), Color::from_argb(0)); // Unset pixel
    }

    #[test]
    fn framebuffer_bounds_check() {
        let mut fb = Framebuffer::new(10, 10);
        // Setting out of bounds should do nothing (not panic)
        fb.set_pixel(100, 100, Color::RED);
        // Getting out of bounds should return transparent
        assert_eq!(fb.get_pixel(100, 100), Color::TRANSPARENT);
    }

    #[test]
    fn blit_image_opaque() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);

        // 2x2 red image (RGBA)
        let image = DecodedImage::new(
            2,
            2,
            PixelFormat::Rgba,
            vec![
                255, 0, 0, 255, // red
                255, 0, 0, 255, // red
                255, 0, 0, 255, // red
                255, 0, 0, 255, // red
            ],
        );

        fb.blit_image(3, 3, &image);

        // Check image pixels
        assert_eq!(fb.get_pixel(3, 3), Color::RED);
        assert_eq!(fb.get_pixel(4, 3), Color::RED);
        assert_eq!(fb.get_pixel(3, 4), Color::RED);
        assert_eq!(fb.get_pixel(4, 4), Color::RED);

        // Check surrounding pixels are still white
        assert_eq!(fb.get_pixel(2, 2), Color::WHITE);
        assert_eq!(fb.get_pixel(5, 5), Color::WHITE);
    }

    #[test]
    fn blit_image_transparent() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::RED);

        // 2x2 fully transparent image
        let image = DecodedImage::new(
            2,
            2,
            PixelFormat::Rgba,
            vec![
                0, 0, 255, 0, // transparent blue
                0, 0, 255, 0, // transparent blue
                0, 0, 255, 0, // transparent blue
                0, 0, 255, 0, // transparent blue
            ],
        );

        fb.blit_image(3, 3, &image);

        // Transparent pixels should not change the framebuffer
        assert_eq!(fb.get_pixel(3, 3), Color::RED);
        assert_eq!(fb.get_pixel(4, 4), Color::RED);
    }

    #[test]
    fn blit_image_alpha_blend() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE); // RGB: (255, 255, 255)

        // 1x1 semi-transparent black (50% alpha)
        let image = DecodedImage::new(
            1,
            1,
            PixelFormat::Rgba,
            vec![0, 0, 0, 128], // black with ~50% alpha
        );

        fb.blit_image(5, 5, &image);

        // Result should be approximately gray
        let result = fb.get_pixel(5, 5);
        // 0 * 128/255 + 255 * (255-128)/255 = 127
        assert_eq!(result.r, 127);
        assert_eq!(result.g, 127);
        assert_eq!(result.b, 127);
    }

    #[test]
    fn blit_image_clipping_negative() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);

        // 4x4 red image, blitted at (-2, -2) - only 2x2 should be visible
        let mut data = Vec::with_capacity(64);
        for _ in 0..16 {
            data.extend_from_slice(&[255, 0, 0, 255]); // red
        }
        let image = DecodedImage::new(4, 4, PixelFormat::Rgba, data);

        fb.blit_image(-2, -2, &image);

        // Visible part (0,0 to 1,1) should be red
        assert_eq!(fb.get_pixel(0, 0), Color::RED);
        assert_eq!(fb.get_pixel(1, 1), Color::RED);

        // Outside visible part should be white
        assert_eq!(fb.get_pixel(2, 2), Color::WHITE);
    }

    #[test]
    fn blit_image_clipping_overflow() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);

        // 4x4 blue image, blitted at (8, 8) - only 2x2 should be visible
        let mut data = Vec::with_capacity(64);
        for _ in 0..16 {
            data.extend_from_slice(&[0, 0, 255, 255]); // blue
        }
        let image = DecodedImage::new(4, 4, PixelFormat::Rgba, data);

        fb.blit_image(8, 8, &image);

        // Visible part (8,8 to 9,9) should be blue
        assert_eq!(fb.get_pixel(8, 8), Color::BLUE);
        assert_eq!(fb.get_pixel(9, 9), Color::BLUE);

        // Outside visible part should be white
        assert_eq!(fb.get_pixel(7, 7), Color::WHITE);
    }

    #[test]
    fn blit_image_rgb_format() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);

        // RGB image (no alpha) - should be treated as opaque
        let image = DecodedImage::new(
            2,
            1,
            PixelFormat::Rgb,
            vec![
                0, 255, 0, // green
                0, 0, 255, // blue
            ],
        );

        fb.blit_image(0, 0, &image);

        assert_eq!(fb.get_pixel(0, 0), Color::GREEN);
        assert_eq!(fb.get_pixel(1, 0), Color::BLUE);
    }

    #[test]
    fn blit_image_empty() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear(Color::WHITE);

        // Empty image
        let image = DecodedImage::new(0, 0, PixelFormat::Rgba, vec![]);

        // Should not panic
        fb.blit_image(5, 5, &image);

        // Framebuffer should be unchanged
        assert_eq!(fb.get_pixel(5, 5), Color::WHITE);
    }

    #[test]
    fn alpha_blend_opaque_over_color() {
        // Opaque red over white should give red
        let result = alpha_blend(Color::rgba(255, 0, 0, 255), Color::WHITE);
        assert_eq!(result, Color::RED);
    }

    #[test]
    fn alpha_blend_50_percent() {
        // 50% red over white
        let result = alpha_blend(Color::rgba(255, 0, 0, 128), Color::WHITE);
        // R: 255*128/255 + 255*127/255 = 128 + 127 = 255... wait
        // Actually: R: (255 * 128 + 255 * 127) / 255 = (32640 + 32385) / 255 = 255
        // Hmm, let me recalculate:
        // src.r * alpha + dst.r * inv_alpha / 255
        // = (255 * 128 + 255 * 127) / 255
        // = 255 * 255 / 255 = 255
        // For G and B: (0 * 128 + 255 * 127) / 255 = 127
        assert_eq!(result.r, 255);
        assert_eq!(result.g, 127);
        assert_eq!(result.b, 127);
    }
}
