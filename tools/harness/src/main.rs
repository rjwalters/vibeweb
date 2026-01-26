//! Test harness for golden tests and page rendering.
//!
//! This tool provides CLI commands for testing and debugging the vibeweb browser.
//!
//! ## Commands
//!
//! - `dom <file>` - Parse an HTML file and display the DOM tree
//! - `dom --stdin` - Parse HTML from standard input
//! - `generate <fixture>` - Generate golden output JSON for a fixture
//! - `compare <fixture>` - Compare current output vs golden file
//! - `golden` - Run all golden tests
//! - `golden --update` - Regenerate all golden files
//!
//! ## Examples
//!
//! ```bash
//! # Parse a local HTML file
//! cargo run -p harness -- dom tools/fixtures/minimal.html
//!
//! # Generate golden output
//! cargo run -p harness -- generate tools/fixtures/basic/empty.html
//!
//! # Compare against golden
//! cargo run -p harness -- compare tools/fixtures/basic/empty.html
//!
//! # Run all golden tests
//! cargo run -p harness -- golden
//!
//! # Update all golden files
//! cargo run -p harness -- golden --update
//! ```

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

use serde::{Deserialize, Serialize};
use vw_html::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(0);
    }

    match args[1].as_str() {
        "dom" => cmd_dom(&args[2..]),
        "generate" => cmd_generate(&args[2..]),
        "compare" => cmd_compare(&args[2..]),
        "golden" => cmd_golden(&args[2..]),
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
    println!("    dom <file>           Parse HTML file and display DOM tree");
    println!("    dom --stdin          Parse HTML from standard input");
    println!("    generate <fixture>   Generate golden output JSON for fixture");
    println!("    compare <fixture>    Compare current vs golden output");
    println!("    golden               Run all golden tests");
    println!("    golden --update      Regenerate all golden files");
    println!("    help                 Show this help message");
    println!();
    println!("OPTIONS:");
    println!("    --compact            Use compact single-line output format (dom only)");
    println!("    --update             Regenerate golden files (golden only)");
    println!();
    println!("EXAMPLES:");
    println!("    harness dom tools/fixtures/minimal.html");
    println!("    harness generate tools/fixtures/basic/empty.html");
    println!("    harness compare tools/fixtures/basic/empty.html");
    println!("    harness golden");
    println!("    harness golden --update");
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

/// Golden file output format - DOM tree structure in JSON
#[derive(Debug, Serialize, Deserialize)]
struct GoldenOutput {
    /// Version of golden format (for future compatibility)
    version: u32,
    /// Compact DOM representation
    dom_compact: String,
    /// Full DOM tree representation
    dom_tree: String,
}

impl GoldenOutput {
    fn from_html(html: &str) -> Self {
        let document = parse(html);
        GoldenOutput {
            version: 1,
            dom_compact: document.debug_compact(),
            dom_tree: document.debug_tree(),
        }
    }

    fn compare(&self, other: &GoldenOutput) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.version != other.version {
            errors.push(format!(
                "Version mismatch: expected {}, got {}",
                other.version, self.version
            ));
        }

        if self.dom_compact != other.dom_compact {
            errors.push(format!(
                "DOM compact output differs:\nExpected:\n{}\n\nGot:\n{}",
                other.dom_compact, self.dom_compact
            ));
        }

        if self.dom_tree != other.dom_tree {
            errors.push(format!(
                "DOM tree output differs:\nExpected:\n{}\n\nGot:\n{}",
                other.dom_tree, self.dom_tree
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Get the golden file path for a fixture
fn golden_path(fixture_path: &Path) -> PathBuf {
    let fixture_dir = fixture_path
        .parent()
        .expect("Fixture must have parent directory");

    // Navigate to tools/fixtures/expected/
    let base = fixture_dir
        .ancestors()
        .find(|p| p.ends_with("tools/fixtures"))
        .expect("Fixture must be under tools/fixtures/");

    let expected_dir = base.join("expected");

    // Get relative path from fixtures/ to the file
    let rel_path = fixture_path
        .strip_prefix(base)
        .expect("Should be able to strip prefix");

    // Create golden filename: change .html to .json
    let mut golden_name = rel_path.to_path_buf();
    golden_name.set_extension("json");

    expected_dir.join(golden_name)
}

/// Handles the `generate` command - create golden output file
fn cmd_generate(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No fixture file specified");
        eprintln!();
        eprintln!("Usage: harness generate <fixture.html>");
        process::exit(1);
    }

    let fixture_path = Path::new(&args[0]);
    if !fixture_path.exists() {
        eprintln!("Error: Fixture file not found: {}", fixture_path.display());
        process::exit(1);
    }

    let html = fs::read_to_string(fixture_path).unwrap_or_else(|e| {
        eprintln!("Failed to read fixture '{}': {}", fixture_path.display(), e);
        process::exit(1);
    });

    let golden = GoldenOutput::from_html(&html);
    let golden_file = golden_path(fixture_path);

    // Create parent directories
    if let Some(parent) = golden_file.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("Failed to create directory '{}': {}", parent.display(), e);
            process::exit(1);
        });
    }

    // Write golden file
    let json = serde_json::to_string_pretty(&golden).unwrap();
    fs::write(&golden_file, json).unwrap_or_else(|e| {
        eprintln!(
            "Failed to write golden file '{}': {}",
            golden_file.display(),
            e
        );
        process::exit(1);
    });

    println!("✓ Generated golden file: {}", golden_file.display());
}

/// Handles the `compare` command - compare current vs golden output
fn cmd_compare(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No fixture file specified");
        eprintln!();
        eprintln!("Usage: harness compare <fixture.html>");
        process::exit(1);
    }

    let fixture_path = Path::new(&args[0]);
    if !fixture_path.exists() {
        eprintln!("Error: Fixture file not found: {}", fixture_path.display());
        process::exit(1);
    }

    let golden_file = golden_path(fixture_path);
    if !golden_file.exists() {
        eprintln!("Error: Golden file not found: {}", golden_file.display());
        eprintln!(
            "Run 'harness generate {}' to create it",
            fixture_path.display()
        );
        process::exit(1);
    }

    // Read fixture
    let html = fs::read_to_string(fixture_path).unwrap_or_else(|e| {
        eprintln!("Failed to read fixture '{}': {}", fixture_path.display(), e);
        process::exit(1);
    });

    // Read golden file
    let golden_json = fs::read_to_string(&golden_file).unwrap_or_else(|e| {
        eprintln!(
            "Failed to read golden file '{}': {}",
            golden_file.display(),
            e
        );
        process::exit(1);
    });

    let expected: GoldenOutput = serde_json::from_str(&golden_json).unwrap_or_else(|e| {
        eprintln!(
            "Failed to parse golden file '{}': {}",
            golden_file.display(),
            e
        );
        process::exit(1);
    });

    // Generate current output
    let actual = GoldenOutput::from_html(&html);

    // Compare
    match actual.compare(&expected) {
        Ok(()) => {
            println!("✓ PASS: {}", fixture_path.display());
            process::exit(0);
        }
        Err(errors) => {
            println!("✗ FAIL: {}", fixture_path.display());
            for error in errors {
                println!("{}", error);
            }
            process::exit(1);
        }
    }
}

/// Find all fixture HTML files under tools/fixtures/
fn find_fixtures() -> Vec<PathBuf> {
    let fixtures_dir = Path::new("tools/fixtures");
    if !fixtures_dir.exists() {
        eprintln!(
            "Error: fixtures directory not found: {}",
            fixtures_dir.display()
        );
        process::exit(1);
    }

    let mut fixtures = Vec::new();
    find_fixtures_recursive(fixtures_dir, &mut fixtures);
    fixtures.sort();
    fixtures
}

fn find_fixtures_recursive(dir: &Path, fixtures: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap() != "expected" {
                find_fixtures_recursive(&path, fixtures);
            } else if path.extension().and_then(|s| s.to_str()) == Some("html") {
                fixtures.push(path);
            }
        }
    }
}

/// Handles the `golden` command - run all golden tests or update them
fn cmd_golden(args: &[String]) {
    let update = args.iter().any(|a| a == "--update");

    let fixtures = find_fixtures();
    if fixtures.is_empty() {
        eprintln!("No fixture files found under tools/fixtures/");
        process::exit(1);
    }

    if update {
        println!("Updating {} golden file(s)...", fixtures.len());
        let mut updated = 0;
        let mut failed = 0;

        for fixture in &fixtures {
            let html = match fs::read_to_string(fixture) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("✗ Failed to read {}: {}", fixture.display(), e);
                    failed += 1;
                    continue;
                }
            };

            let golden = GoldenOutput::from_html(&html);
            let golden_file = golden_path(fixture);

            // Create parent directories
            if let Some(parent) = golden_file.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("✗ Failed to create directory {}: {}", parent.display(), e);
                    failed += 1;
                    continue;
                }
            }

            // Write golden file
            let json = serde_json::to_string_pretty(&golden).unwrap();
            match fs::write(&golden_file, json) {
                Ok(()) => {
                    println!("✓ Updated: {}", fixture.display());
                    updated += 1;
                }
                Err(e) => {
                    eprintln!("✗ Failed to write {}: {}", golden_file.display(), e);
                    failed += 1;
                }
            }
        }

        println!();
        println!("Summary: {} updated, {} failed", updated, failed);
        if failed > 0 {
            process::exit(1);
        }
    } else {
        println!("Running {} golden test(s)...", fixtures.len());
        let mut passed = 0;
        let mut failed = 0;
        let mut missing = 0;

        for fixture in &fixtures {
            let golden_file = golden_path(fixture);
            if !golden_file.exists() {
                println!("⚠ SKIP: {} (no golden file)", fixture.display());
                missing += 1;
                continue;
            }

            // Read fixture
            let html = match fs::read_to_string(fixture) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("✗ FAIL: {} (read error: {})", fixture.display(), e);
                    failed += 1;
                    continue;
                }
            };

            // Read golden file
            let golden_json = match fs::read_to_string(&golden_file) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("✗ FAIL: {} (golden read error: {})", fixture.display(), e);
                    failed += 1;
                    continue;
                }
            };

            let expected: GoldenOutput = match serde_json::from_str(&golden_json) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("✗ FAIL: {} (golden parse error: {})", fixture.display(), e);
                    failed += 1;
                    continue;
                }
            };

            // Generate current output
            let actual = GoldenOutput::from_html(&html);

            // Compare
            match actual.compare(&expected) {
                Ok(()) => {
                    println!("✓ PASS: {}", fixture.display());
                    passed += 1;
                }
                Err(_errors) => {
                    println!("✗ FAIL: {}", fixture.display());
                    // Don't print full diff in summary mode
                    failed += 1;
                }
            }
        }

        println!();
        println!(
            "Summary: {} passed, {} failed, {} missing",
            passed, failed, missing
        );
        println!("Total: {} fixtures", fixtures.len());

        if missing > 0 {
            println!();
            println!("Run 'harness golden --update' to generate missing golden files");
        }

        if failed > 0 {
            process::exit(1);
        }
    }
}
