//! Vibeweb browser - main entry point
//!
//! M4 milestone: Uses the vw-renderer crate to orchestrate the render pipeline.

use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use vw_gfx::Framebuffer;
use vw_platform::{Event, Window, WindowConfig, WinitWindow};
use vw_renderer::Browser;

/// Graphics state that holds the softbuffer context and surface.
/// The context must be kept alive for the surface to work.
struct GraphicsState {
    #[allow(dead_code)] // Context must be kept alive for surface
    context: Context<Arc<WinitWindow>>,
    surface: Surface<Arc<WinitWindow>, Arc<WinitWindow>>,
}

/// Default HTML to render when no document is loaded.
const DEFAULT_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Vibeweb Browser</title>
</head>
<body>
    <h1>Welcome to Vibeweb</h1>
    <p>A browser built from scratch in Rust.</p>
    <div>
        <p>This is the render pipeline test page.</p>
    </div>
</body>
</html>
"#;

/// Default CSS for the default document.
const DEFAULT_CSS: &str = "";

fn main() {
    println!("vibeweb browser - M4 render pipeline");

    // Create window configuration
    let config = WindowConfig::new("Vibeweb Browser", 800, 600);

    // Create the window
    let window = Window::new(config).expect("Failed to create window");

    // State for rendering
    let mut framebuffer = Framebuffer::new(800, 600);
    let mut graphics: Option<GraphicsState> = None;

    // Create the browser with a default document
    let mut browser =
        Browser::new(DEFAULT_HTML, DEFAULT_CSS, 800, 600).expect("Failed to create browser");

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

                    // Resize browser viewport if window size changed
                    browser.resize(width, height);

                    // Ensure framebuffer matches window size
                    if framebuffer.width() != width || framebuffer.height() != height {
                        framebuffer.resize(width, height);
                    }

                    // Paint using the render pipeline
                    browser.paint(&mut framebuffer);

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
                    // Browser.resize handles invalidation
                    browser.resize(width, height);
                    ctx.request_redraw();
                }

                Event::CloseRequested => {
                    println!("Window close requested, exiting...");
                }

                // Input events (will be handled in M5 - Navigation UX)
                Event::MouseMoved { .. }
                | Event::MouseButton { .. }
                | Event::MouseWheel { .. }
                | Event::KeyboardInput { .. }
                | Event::CharacterInput { .. } => {
                    // TODO: Handle input events for interactive features
                }
            }
        })
        .expect("Event loop error");
}
