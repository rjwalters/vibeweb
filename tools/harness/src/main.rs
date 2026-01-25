//! Test harness for golden tests and page rendering.
//!
//! This tool provides CLI commands for testing and debugging the vibeweb browser.
//!
//! ## Commands
//!
//! - `dom <file>` - Parse an HTML file and display the DOM tree
//! - `dom --stdin` - Parse HTML from standard input
//!
//! ## Examples
//!
//! ```bash
//! # Parse a local HTML file
//! cargo run -p harness -- dom tools/fixtures/minimal.html
//!
//! # Parse HTML from stdin
//! echo "<p>test</p>" | cargo run -p harness -- dom --stdin
//!
//! # Compact output format
//! cargo run -p harness -- dom --compact tools/fixtures/minimal.html
//! ```

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

use vw_html::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(0);
    }

    match args[1].as_str() {
        "dom" => cmd_dom(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!();
            print_usage();
            process::exit(1);
        }
    }
}

/// Prints usage information.
fn print_usage() {
    println!("vibeweb test harness");
    println!();
    println!("USAGE:");
    println!("    harness <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    dom <file>       Parse HTML file and display DOM tree");
    println!("    dom --stdin      Parse HTML from standard input");
    println!("    help             Show this help message");
    println!();
    println!("OPTIONS:");
    println!("    --compact        Use compact single-line output format");
    println!();
    println!("EXAMPLES:");
    println!("    harness dom tools/fixtures/minimal.html");
    println!("    echo \"<p>test</p>\" | harness dom --stdin");
    println!("    harness dom --compact tools/fixtures/minimal.html");
}

/// Handles the `dom` command - parse HTML and display DOM tree.
fn cmd_dom(args: &[String]) {
    let mut compact = false;
    let mut source: Option<String> = None;
    let mut use_stdin = false;

    // Parse arguments
    for arg in args {
        match arg.as_str() {
            "--compact" | "-c" => compact = true,
            "--stdin" | "-" => use_stdin = true,
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                process::exit(1);
            }
            _ => source = Some(arg.clone()),
        }
    }

    // Get the HTML content
    let html = if use_stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    } else if let Some(path) = source {
        fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read file '{}': {}", path, e);
            process::exit(1);
        })
    } else {
        eprintln!("Error: No input file specified");
        eprintln!();
        eprintln!("Usage: harness dom <file>");
        eprintln!("       harness dom --stdin");
        process::exit(1);
    };

    // Parse and display
    let document = parse(&html);

    if compact {
        println!("{}", document.debug_compact());
    } else {
        print!("{}", document.debug_tree());
    }
}
