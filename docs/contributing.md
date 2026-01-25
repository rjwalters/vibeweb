# Contributing

## Getting Started

1. Clone the repository
2. Ensure you have Rust stable installed
3. Run `cargo build --workspace` to verify setup
4. Pick an issue labeled `good-first-crate:<name>`

## Development Commands

```bash
# Format code
cargo fmt

# Run lints
cargo clippy --all-targets --all-features

# Run all tests
cargo test --workspace

# Run a specific crate's tests
cargo test -p vw-html
```

## Pull Request Guidelines

Every PR should:

1. **Include tests** where practical
2. **Add trace/debug output** if it improves observability
3. **Update `docs/architecture.md`** if it changes interfaces
4. **Keep changes focused** — one logical change per PR

## Commit Messages

Use conventional commit style:
- `feat(html): add tokenizer for basic tags`
- `fix(layout): correct inline box width calculation`
- `test(css): add selector specificity tests`
- `docs: update architecture diagram`

## Code Style

- Follow `rustfmt` defaults
- Prefer explicit over implicit
- No hidden global state
- Document public APIs
- Add `#[cfg(test)]` modules for unit tests

## Issue Guidelines

When creating issues, define:
- **Inputs/outputs** — what data flows in and out
- **Success criteria** — how we know it's done
- **Tests or demo** — verification method
- **Crate scope** — which crate(s) are affected

## Definition of Done (per layer)

### Parsers (HTML, CSS)
- Passes fuzz corpus without panics
- Handles malformed input gracefully
- Has trace dump output for debugging

### Layout
- Deterministic output for same input
- Golden tests for key fixtures
- Handles edge cases (empty nodes, deep nesting)

### Rendering
- Consistent across runs
- No visual artifacts in test fixtures
- Performance within budget
