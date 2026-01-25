# Rust Browser From Scratch (Stress Test)

A deliberately ambitious project to build a **functional web browser in Rust**, from first principles, with as few external dependencies as practical.

This repository also doubles as a **stress test for an agent orchestration system**: the codebase, issues, and milestones are structured to maximize parallelizable work and to expose coordination failures early.

> Goal: render real-world pages (at least a curated subset) end-to-end: **URL → network → parsing → layout → paint → input → navigation**.

---

## What "functional" means

A browser is "functional" for this repo if it can:

- Load `http://` and `https://` URLs
- Parse HTML into a DOM
- Parse CSS (enough for layout + styling)
- Compute layout for block + inline text + basic positioning
- Paint into a window
- Support scrolling, resizing, clicking links, back/forward, and reload
- Render a curated set of target pages consistently

Everything beyond that is "bonus."

---

## Non-goals (at least initially)

To keep the scope survivable:

- ✅ No JavaScript at first (later: a tiny JS subset or embedded engine)
- ✅ No full HTML/CSS spec compliance
- ✅ No extensions, sync, devtools, accessibility, printing
- ✅ No video/audio playback
- ✅ No WebGPU/WebGL (later maybe)

---

## Project structure

The architecture is intentionally split into crates so work can proceed in parallel.

```
crates/
  browser/        # app shell, tabs, history, UI integration
  net/            # URL fetch, redirects, caching, cookies (later)
  tls/            # TLS glue layer (minimal wrapper over a vetted impl)
  html/           # tokenizer + tree builder (DOM)
  dom/            # DOM types, traversal, mutation APIs (minimal)
  css/            # CSS tokenizer/parser + selector matching
  style/          # cascade, computed styles
  layout/         # layout tree, block/inline layout, text measurement
  gfx/            # display list, rasterization
  platform/       # window, input, timers, clipboard (thin abstraction)
  image/          # png/jpeg decoding (later: via minimal deps)
  fonts/          # font loading, shaping (later: minimal path first)
tools/
  harness/        # golden tests, page fetch+render diffing
  fixtures/       # test pages, expected rendering data
docs/
  roadmap.md
  architecture.md
  contributing.md
```

---

## Milestones

Each milestone should end with a demo artifact (GIF/video) and a reproducible command.

### M0 — Bootstrap
- Workspace builds, CI runs, basic window opens
- A single "hello triangle / hello rect" render path

### M1 — Networking + HTML
- Fetch a URL over HTTP
- Parse HTML into a DOM
- Display a DOM tree viewer (text debug mode)

### M2 — CSS + Style
- Parse a CSS stylesheet
- Match selectors against DOM
- Compute styles for nodes

### M3 — Layout + Text
- Block layout, basic inline text
- Line breaking (simple), font metrics (minimal)
- Viewport resize + scroll

### M4 — Painting
- Display list generation
- Rasterize to window
- Backgrounds, borders, text, simple images

### M5 — Navigation UX
- URL bar, back/forward/reload
- Link clicking + basic focus
- History model and basic error pages

### M6 — "Real page" target set
- Render a curated set of ~20 pages (mostly static)
- Golden tests for layout/paint output
- Performance budgets and crash-free runs

---

## Target pages (starter set)

We maintain a curated set of pages designed to steadily increase complexity.

- `about:blank`
- Minimal HTML fixtures in `tools/fixtures/`
- Then progressively:
  - Wikipedia article (mostly static)
  - MDN docs page (static-ish)
  - A simple blog with images + CSS
  - One "nasty" page with lots of nested layout

A page is considered "passing" if it:
- Loads without panic
- Produces deterministic layout metrics
- Matches golden render within tolerance (or matches known limitations)

---

## Design principles

- **Determinism first**: stable output > correctness
- **Small, explicit interfaces** between crates
- **No hidden global state** (avoid singletons)
- **Make it testable**: every layer has unit tests + integration tests
- **Traceability**: structured logging + "explain" dumps (DOM, style tree, layout tree, display list)

---

## Safety & dependency policy

We're building from scratch, but not trying to reinvent cryptography.

- Rust stable
- Prefer minimal, well-audited dependencies where appropriate (e.g. TLS)
- Unsafe code allowed only behind clearly documented modules with tests
- Fuzzing targets for parsers (HTML/CSS)

---

## Quickstart (eventually)

Once M0 lands:

```bash
cargo run -p browser
```

Example usage:

```bash
cargo run -p browser -- https://example.com
```

---

## Development workflow

### Commands

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test --workspace
```

### Golden rendering tests (planned)

```bash
cargo run -p harness -- render --url https://example.com --out artifacts/example.png
cargo test -p harness -- golden
```

---

## Contributing

This repo is intentionally structured for parallel work.

* Pick a crate + an issue labeled `good-first-crate:<name>`
* Every PR should:

  * Include tests where practical
  * Add a trace dump or debug view if it improves observability
  * Update `docs/architecture.md` if it changes interfaces

See [`docs/contributing.md`](docs/contributing.md).

---

## Coordination model (for agent orchestration)

We treat each crate like a "service boundary."

* Each issue should define:

  * Inputs/outputs
  * Success criteria
  * Tests or demo harness
* Prefer "vertical slices" that connect two layers end-to-end
* Avoid giant "implement CSS" issues — split by features + fixtures

---

## Inspiration / references

This project is inspired by:

* Servo (Rust browser engine research)
* "toy" browser implementations and specs, used as references (not copied)

We'll list concrete references in `docs/roadmap.md` as they're adopted.

---

## License

MIT OR Apache-2.0 (dual-licensed), unless stated otherwise.
