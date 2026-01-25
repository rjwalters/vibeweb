//! Vibeweb browser - main entry point
//!
//! M0 milestone: Opens a window and displays a colored rectangle.

use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use vw_gfx::{Color, Framebuffer, Rect};
use vw_platform::{Event, Window, WindowConfig, WinitWindow};

/// Graphics state that holds the softbuffer context and surface.
/// The context must be kept alive for the surface to work.
struct GraphicsState {
    #[allow(dead_code)] // Context must be kept alive for surface
    context: Context<Arc<WinitWindow>>,
    surface: Surface<Arc<WinitWindow>, Arc<WinitWindow>>,
}

fn main() {
    println!("vibeweb browser - M0 bootstrap");

    // Create window configuration
    let config = WindowConfig::new("Vibeweb Browser", 800, 600);

    // Create the window
    let window = Window::new(config).expect("Failed to create window");

    // State for rendering
    let mut framebuffer = Framebuffer::new(800, 600);
    let mut graphics: Option<GraphicsState> = None;

    // Run the event loop
    window
        .run(move |ctx, event| {
            // Initialize graphics on first event
            if graphics.is_none() {
                let win = Arc::clone(ctx.window());
                let context = Context::new(win.clone()).expect("Failed to create context");
                let surface = Surface::new(&context, win).expect("Failed to create surface");
                graphics = Some(GraphicsState { context, surface });
                ctx.request_redraw();
            }

            match event {
                Event::Redraw => {
                    let (width, height) = ctx.size();

                    // Ensure framebuffer matches window size
                    if framebuffer.width() != width || framebuffer.height() != height {
                        framebuffer.resize(width, height);
                    }

                    // Clear to white background
                    framebuffer.clear(Color::WHITE);

                    // Calculate centered rectangle position
                    let rect_width = 200u32;
                    let rect_height = 200u32;
                    let rect_x = (width.saturating_sub(rect_width) / 2) as i32;
                    let rect_y = (height.saturating_sub(rect_height) / 2) as i32;

                    // Draw a centered red rectangle
                    let rect = Rect::new(rect_x, rect_y, rect_width, rect_height);
                    framebuffer.fill_rect(rect, Color::RED);

                    // Present to window via softbuffer
                    if let Some(ref mut gfx) = graphics {
                        let _ = gfx.surface.resize(
                            NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN),
                            NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN),
                        );

                        if let Ok(mut buffer) = gfx.surface.buffer_mut() {
                            buffer.copy_from_slice(framebuffer.pixels());
                            let _ = buffer.present();
                        }
                    }
                }

                Event::Resize { width, height } => {
                    framebuffer.resize(width, height);
                    ctx.request_redraw();
                }

                Event::CloseRequested => {
                    println!("Window close requested, exiting...");
                }
            }
        })
        .expect("Event loop error");
}
