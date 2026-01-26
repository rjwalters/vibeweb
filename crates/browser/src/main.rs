//! Vibeweb browser - main entry point
//!
//! M5 milestone: URL loading and navigation support

use softbuffer::{Context, Surface};
use std::env;
use std::fs;
use std::io;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vw_gfx::Framebuffer;
use vw_net::{Client, NetError};
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

/// Represents the source of a document to load
#[derive(Debug, Clone)]
enum DocumentSource {
    /// Load from a URL (http:// or https://)
    Url(String),
    /// Load from a local file path
    File(PathBuf),
    /// Use the default hardcoded HTML
    Default,
}

/// Errors that can occur during document loading
#[derive(Debug)]
enum LoadError {
    /// Network error (connection failed, timeout, etc.)
    Network(NetError),
    /// File I/O error
    Io(io::Error),
    /// Invalid UTF-8 in loaded content
    InvalidUtf8(std::string::FromUtf8Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Network(e) => write!(f, "Network error: {}", e),
            LoadError::Io(e) => write!(f, "File I/O error: {}", e),
            LoadError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
        }
    }
}

impl From<NetError> for LoadError {
    fn from(e: NetError) -> Self {
        LoadError::Network(e)
    }
}

impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> Self {
        LoadError::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for LoadError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        LoadError::InvalidUtf8(e)
    }
}

impl DocumentSource {
    /// Determine the document source from a command-line argument
    fn from_arg(arg: &str) -> Self {
        if arg.starts_with("http://") || arg.starts_with("https://") {
            DocumentSource::Url(arg.to_string())
        } else {
            DocumentSource::File(PathBuf::from(arg))
        }
    }

    /// Load the document from this source
    ///
    /// Returns (html, css) tuple. Currently CSS is always empty string
    /// as external stylesheet loading is not yet implemented.
    fn load(&self) -> Result<(String, String), LoadError> {
        match self {
            DocumentSource::Url(url) => load_url(url),
            DocumentSource::File(path) => load_file(path),
            DocumentSource::Default => Ok((DEFAULT_HTML.to_string(), DEFAULT_CSS.to_string())),
        }
    }
}

/// Load a document from a URL
fn load_url(url: &str) -> Result<(String, String), LoadError> {
    println!("Loading URL: {}", url);

    let client = Client::new();
    let response = client.fetch(url)?;

    // Convert body to UTF-8 string, using lossy conversion for invalid UTF-8
    let html = String::from_utf8_lossy(&response.body).to_string();

    // CSS will be extracted from <style> tags by Document::load()
    let css = String::new(); // External CSS via <link> tags not yet supported

    println!("Loaded {} bytes from {}", html.len(), url);
    Ok((html, css))
}

/// Load a document from a local file
fn load_file(path: &Path) -> Result<(String, String), LoadError> {
    println!("Loading file: {}", path.display());

    let html = fs::read_to_string(path)?;

    // TODO: Look for companion .css file or extract inline styles
    // CSS will be extracted from <style> tags by Document::load()
    let css = String::new(); // Companion .css files not yet supported
    println!("Loaded {} bytes from {}", html.len(), path.display());
    Ok((html, css))
}

fn main() {
    println!("vibeweb browser - M5 URL loading and navigation");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let source = match args.get(1) {
        Some(arg) => DocumentSource::from_arg(arg),
        None => DocumentSource::Default,
    };

    // Load the document
    let (html, css) = match source.load() {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error loading document: {}", e);
            eprintln!("Falling back to default page");
            (DEFAULT_HTML.to_string(), DEFAULT_CSS.to_string())
        }
    };

    // Create window configuration
    let config = WindowConfig::new("Vibeweb Browser", 800, 600);

    // Create the window
    let window = Window::new(config).expect("Failed to create window");

    // State for rendering
    let mut framebuffer = Framebuffer::new(800, 600);
    let mut graphics: Option<GraphicsState> = None;

    // Create the browser with the loaded document
    let mut browser = Browser::new(&html, &css, 800, 600).expect("Failed to create browser");

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

                // Mouse wheel events for scrolling
                Event::MouseWheel { delta_x, delta_y } => {
                    // Apply scroll delta to browser
                    // Note: delta is typically negative when scrolling down (content moves up)
                    // We negate delta_y so positive delta scrolls content up (natural scrolling)
                    browser.scroll(delta_x as f32, -delta_y as f32);
                    ctx.request_redraw();
                }

                // Other input events (will be handled in future M5 work)
                Event::MouseMoved { .. }
                | Event::MouseButton { .. }
                | Event::KeyboardInput { .. }
                | Event::CharacterInput { .. } => {
                    // TODO: Handle input events for interactive features (link clicking, etc.)
                }
            }
        })
        .expect("Event loop error");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_source_from_arg_http() {
        let source = DocumentSource::from_arg("http://example.com");
        assert!(matches!(source, DocumentSource::Url(_)));
    }

    #[test]
    fn test_document_source_from_arg_https() {
        let source = DocumentSource::from_arg("https://example.com");
        assert!(matches!(source, DocumentSource::Url(_)));
    }

    #[test]
    fn test_document_source_from_arg_file() {
        let source = DocumentSource::from_arg("test.html");
        assert!(matches!(source, DocumentSource::File(_)));
    }

    #[test]
    fn test_document_source_from_arg_file_with_slash() {
        let source = DocumentSource::from_arg("./test.html");
        assert!(matches!(source, DocumentSource::File(_)));
    }

    #[test]
    fn test_load_default() {
        let source = DocumentSource::Default;
        let result = source.load();
        assert!(result.is_ok());
        let (html, css) = result.unwrap();
        assert!(html.contains("Welcome to Vibeweb"));
        assert_eq!(css, "");
    }
}
