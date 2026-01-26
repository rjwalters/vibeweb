# Roadmap

## Completed Milestones

### M0 — Bootstrap ✓
- [x] Workspace builds successfully
- [x] CI pipeline runs (GitHub Actions)
- [x] Basic window opens via platform crate
- [x] "Hello rect" render path works

### M1 — Networking + HTML ✓
- [x] Fetch a URL over HTTP/HTTPS
- [x] Parse HTML into a DOM
- [x] Display a DOM tree viewer (text debug mode)

### M2 — CSS + Style ✓
- [x] Parse a CSS stylesheet
- [x] Match selectors against DOM
- [x] Compute styles for nodes

### M3 — Layout + Text ✓
- [x] Block layout, basic inline text
- [x] Line breaking (simple), font metrics (minimal)
- [x] Viewport resize + scroll

### M4 — Painting ✓
- [x] Display list generation
- [x] Rasterize to window
- [x] Backgrounds, borders, text rendering

## Current Focus: M5 — Navigation UX

- [x] URL bar input support
- [x] Mouse wheel scrolling
- [x] DOM event propagation foundation
- [x] Viewport scroll offset tracking
- [ ] Link clicking
- [ ] Back/forward navigation
- [ ] Reload functionality
- [ ] History model integration
- [ ] Basic error pages

## Next: M6 — Real Page Target Set

- [ ] Render curated set of ~20 real-world pages
- [ ] Golden tests for layout/paint output
- [ ] Performance budgets and profiling
- [ ] Regression testing infrastructure

## References

- [Loom Orchestration Framework](https://github.com/rjwalters/loom) - AI-powered development workflow automation
- [HTML Living Standard](https://html.spec.whatwg.org/) - HTML parsing reference
- [CSS Snapshot 2023](https://www.w3.org/TR/css-2023/) - CSS specification reference
- [WebRender Architecture](https://github.com/servo/webrender) - Display list and rendering patterns
