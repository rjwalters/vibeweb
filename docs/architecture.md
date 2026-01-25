# Architecture

## Overview

The browser is structured as a collection of independent crates with explicit interfaces between them. This design enables parallel development and testing.

## Crate Dependencies (Planned)

```
browser
├── vw-platform (window, input)
├── vw-gfx (rendering)
├── vw-layout
│   └── vw-style
│       ├── vw-css
│       └── vw-dom
│           └── vw-html
├── vw-net
│   └── vw-tls
├── vw-image
└── vw-fonts
```

## Data Flow

```
URL → net → html → dom → css+style → layout → gfx → platform (window)
                                                ↑
                                            image, fonts
```

## Crate Responsibilities

### browser
Main application shell. Manages tabs, history, URL bar, and coordinates the render pipeline.

### vw-net
HTTP/HTTPS fetching, redirects, connection pooling, caching (later), cookies (later).

### vw-tls
Thin wrapper over a vetted TLS implementation. Isolates crypto dependencies.

### vw-html
HTML tokenizer and tree builder. Produces DOM nodes.

### vw-dom
DOM types, tree structure, traversal, and minimal mutation APIs.

### vw-css
CSS tokenizer, parser, and selector matching.

### vw-style
Cascade algorithm, specificity, computed style resolution.

### vw-layout
Layout tree construction, block/inline layout, text measurement, box model.

### vw-gfx
Display list generation and rasterization.

### vw-platform
Window management, input events, timers, clipboard. Thin abstraction over OS APIs.

### vw-image
Image decoding (PNG, JPEG).

### vw-fonts
Font loading and text shaping.

## Interface Contracts

Each crate boundary should have:
- Clear input/output types
- No hidden global state
- Unit tests for the interface
- Debug/trace dumps for observability
