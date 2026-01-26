# Vibeweb Test Harness

Integration testing tool for the vibeweb browser, providing golden file testing for DOM parsing and rendering.

## Purpose

The harness tool enables regression testing by comparing current parser output against known-good "golden" files. This ensures that changes to the HTML parser, DOM tree building, and future layout/rendering don't introduce unintended regressions.

## Features

- **DOM Inspection**: Parse HTML and display the DOM tree structure
- **Golden File Generation**: Create reference output files from fixtures
- **Golden File Comparison**: Detect differences between current and expected output
- **Batch Testing**: Run all golden tests at once

## Installation

From the workspace root:

```bash
cargo build -p harness
```

## Commands

### `dom` - Inspect DOM Tree

Parse an HTML file and display its DOM tree structure.

**Usage**:
```bash
# Display full DOM tree
harness dom tools/fixtures/basic/empty.html

# Compact single-line format
harness dom --compact tools/fixtures/basic/empty.html

# Parse from stdin
echo "<p>test</p>" | harness dom --stdin
```

### `generate` - Create Golden File

Generate a golden output JSON file for a fixture.

**Usage**:
```bash
harness generate tools/fixtures/basic/empty.html
# Creates: tools/fixtures/expected/basic/empty.json
```

**Golden File Format**:
```json
{
  "version": 1,
  "dom_compact": "(compact DOM representation)",
  "dom_tree": "(full tree representation)"
}
```

### `compare` - Compare Against Golden

Compare current output against the golden file for a fixture.

**Usage**:
```bash
harness compare tools/fixtures/basic/empty.html
# ✓ PASS: tools/fixtures/basic/empty.html (if matches)
# ✗ FAIL: tools/fixtures/basic/empty.html (if differs)
```

**Exit codes**:
- `0` - Test passed (output matches golden)
- `1` - Test failed (output differs from golden)
- `2` - Error (golden file missing, parse error, etc.)

### `golden` - Run All Tests

Run all golden tests found under `tools/fixtures/`.

**Usage**:
```bash
# Run all tests
harness golden

# Update all golden files
harness golden --update
```

**Example output**:
```
Running 7 golden test(s)...
✓ PASS: tools/fixtures/basic/empty.html
✓ PASS: tools/fixtures/basic/single-element.html
✓ PASS: tools/fixtures/basic/nested-divs.html
✓ PASS: tools/fixtures/layout/block-inline.html
✓ PASS: tools/fixtures/layout/multi-paragraph.html
✓ PASS: tools/fixtures/styled/colored-boxes.html
✓ PASS: tools/fixtures/minimal.html

Summary: 7 passed, 0 failed, 0 missing
Total: 7 fixtures
```

## Fixture Organization

Fixtures are organized under `tools/fixtures/` by category:

```
tools/fixtures/
├── basic/              # Simple structural tests
│   ├── empty.html      # Minimal empty document
│   ├── single-element.html
│   └── nested-divs.html
├── layout/             # Layout-specific tests
│   ├── block-inline.html
│   └── multi-paragraph.html
├── styled/             # CSS styling tests
│   ├── colored-boxes.html
│   └── boxes.css
├── expected/           # Golden output files (JSON)
│   ├── basic/
│   │   ├── empty.json
│   │   ├── single-element.json
│   │   └── nested-divs.json
│   ├── layout/
│   │   ├── block-inline.json
│   │   └── multi-paragraph.json
│   └── styled/
│       └── colored-boxes.json
└── minimal.html        # Legacy minimal fixture
```

## Workflow

### Adding a New Test

1. **Create the fixture**:
   ```bash
   cat > tools/fixtures/basic/my-test.html <<'EOF'
   <!DOCTYPE html>
   <html>
   <head><title>My Test</title></head>
   <body><p>Test content</p></body>
   </html>
   EOF
   ```

2. **Generate the golden file**:
   ```bash
   cargo run -p harness -- generate tools/fixtures/basic/my-test.html
   ```

3. **Verify it passes**:
   ```bash
   cargo run -p harness -- compare tools/fixtures/basic/my-test.html
   ```

4. **Commit both files**:
   ```bash
   git add tools/fixtures/basic/my-test.html
   git add tools/fixtures/expected/basic/my-test.json
   git commit -m "Add golden test for my-test"
   ```

### Updating Golden Files After Parser Changes

If you intentionally change the parser behavior:

1. **Verify the changes are correct**:
   ```bash
   # Review what changed
   cargo run -p harness -- compare tools/fixtures/basic/empty.html
   ```

2. **Update golden files**:
   ```bash
   cargo run -p harness -- golden --update
   ```

3. **Review the diffs**:
   ```bash
   git diff tools/fixtures/expected/
   ```

4. **Commit if changes are intentional**:
   ```bash
   git add tools/fixtures/expected/
   git commit -m "Update golden files after parser improvement"
   ```

### Running in CI

Add to your CI pipeline:

```bash
# Build harness
cargo build -p harness

# Run all golden tests
cargo run -p harness -- golden

# Exit code 0 = all passed
# Exit code 1 = one or more failed
```

## Design Principles

### Determinism

Golden files rely on deterministic output. The current implementation tests DOM structure, which is deterministic. Future extensions (layout coordinates, rendering) should:

- Round floating point values to consistent precision
- Sort collections before serialization
- Document any platform-specific behavior
- Use tolerance thresholds for floating point comparisons

### Future Extensions

The harness is designed to be extended:

- **Layout testing**: Add layout tree coordinates to golden files
- **Rendering testing**: Add PNG snapshot comparison
- **Performance testing**: Add timing benchmarks
- **Cross-platform testing**: Document platform-specific golden files

See `tools/fixtures/expected/` for the golden file format. Adding new fields to the JSON schema is backwards-compatible if you preserve the `version` field.

## Troubleshooting

**Golden file not found**:
```bash
# Generate it
harness generate tools/fixtures/basic/my-test.html
```

**Test fails but output looks correct**:
```bash
# Update the golden file
harness generate tools/fixtures/basic/my-test.html

# Or update all golden files
harness golden --update
```

**Many tests failing after a change**:
```bash
# Review what changed
harness compare tools/fixtures/basic/empty.html

# If intentional, update all
harness golden --update
```

## Related Documentation

- Project design principles: `../../README.md`
- M6 milestone: Golden tests for layout/paint output
- Determinism principle: "stable output > correctness"
