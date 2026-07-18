# marco-core

[![Crates.io](https://img.shields.io/crates/v/marco-core.svg)](https://crates.io/crates/marco-core)
[![Docs.rs](https://img.shields.io/docsrs/marco-core)](https://docs.rs/marco-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 1.94.1](https://img.shields.io/badge/rust-1.94.1-orange.svg)](https://www.rust-lang.org)

A Rust Markdown library for applications that need **editor-quality accuracy** — parse, render, and extract intelligence from Markdown.

You may be looking for:

- [API documentation](https://docs.rs/marco-core)
- [Feature flags reference](#feature-flags)
- [Changelog](CHANGELOG.md)
- [Development guide](Documentation/DEVELOPMENT.md)

## Usage

```toml
[dependencies]
marco-core = "1.3"
```

```rust
use marco_core::{parse, render, RenderOptions, sanitize_input, InputSource};

let raw = b"# Hello\n\n**world**";
let clean = sanitize_input(raw, InputSource::File); // strips null bytes, fixes invalid UTF-8
let doc = parse(&clean)?;
let html = render(&doc, &RenderOptions::default())?;
// <h1 id="hello"><a class="marco-heading-anchor" href="#hello">Hello</a></h1>
// <p><strong>world</strong></p>
```

Skip expensive work for render-only pipelines:

```rust
use marco_core::{parse_with_options, render, ParseOptions, RenderOptions};

let opts = ParseOptions {
    track_positions: false, // skip span tracking
    parse_math: false,      // skip math parser
    parse_diagrams: false,  // skip mermaid parser
};
let doc = parse_with_options("## Title", opts)?;
let html = render(&doc, &RenderOptions::default())?;
```

Customize rendering (syntax highlighting is on by default, `"github"` theme):

```rust
use marco_core::RenderOptions;

let opts = RenderOptions {
    syntax_highlighting: true,
    line_numbers: true,
    theme: "base16-ocean.dark".to_string(),
};
```

Editor intelligence:

```rust
use marco_core::{parse, MarkdownIntelligenceProvider};

let md = "# Heading\n\nA paragraph with **bold** text.\n";
let doc = parse(md)?;
let mut provider = MarkdownIntelligenceProvider::new();
provider.update_document(doc);

let highlights = provider.highlights(md); // syntax highlight spans
let diagnostics = provider.diagnostics(); // linting results
let completions = provider.completions(""); // context-aware suggestions
```

## Features

**Parse and render**
- CommonMark — headings, lists, blockquotes, fenced/indented code blocks, links, images, emphasis, inline HTML, autolinks, thematic breaks, link references (full, shortcut, collapsed; Unicode casefold matching)
- GFM — tables, strikethrough (`~~text~~`), task lists, footnotes, autolink literals, alerts/admonitions (`[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`)
- Math — inline `$…$` and display `$$…$$` via KaTeX
- Diagrams — Mermaid fenced blocks
- Marco extensions:
  - *Block:* sliders (`@slidestart`…`@slideend` with `---`/`--` separators and optional timer), tab blocks (`:::tab` / `@tab Name`), headerless tables (separator-first `|---|---|`), definition lists (`Term\n: Definition`)
  - *Inline:* mark/highlight (`==text==`), superscript (`^text^`), subscript (`~text~`), subscript-arrow (`˅text˅`), dash-strikethrough (`--text--`), emoji shortcodes (`:smile:` → 😄), inline footnotes (`text^[note content]`), platform mentions (`@user[github]` / `@user[github](Display Name)`), inline task checkboxes (`[ ]` / `[x]`), custom heading IDs (`## Title {#my-id}`) — all composable and nestable with each other and with standard CommonMark/GFM inline syntax

**Editor intelligence**
- Syntax highlighting — per-node highlight tags; `compute_highlights_with_source` adds extra tags for Marco syntax markers (tab block headers, slider separators)
- Diagnostics/linting — `UnsafeLinkProtocol`, `UnresolvedLinkReference`, and more; filter by severity with `DiagnosticsOptions`
- Autocompletion — context-aware suggestions at cursor position
- Hover — returns info for links, headings, and other nodes at a given `Position`
- TOC extraction (`extract_toc`), Markdown generation (`generate_toc_markdown`), and source insertion (`replace_toc_in_text`)

**Reliability**
- CommonMark spec conformance: 357/652 strict, 98.8% structural compliance
- UTF-8 sanitization (`&[u8]` → `String`) strips null bytes and invalid sequences before parsing
- GFM task list checkboxes and admonitions render as SVG icons (no CSS-only dependency)
- 625 tests — 612 integration, 13 doc/unit — all green (plus 9 more under `--features parallel-parse`)

## When to use marco-core

| Need | Recommended |
|---|---|
| Fast bulk processing (static sites, CI pipelines) | [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark) or [`comrak`](https://crates.io/crates/comrak) |
| Editor intelligence (highlights, linting, completions) | **marco-core** |
| CommonMark + GFM + math + diagrams | **marco-core** |
| Walk or transform the AST | **marco-core** |
| Minimal parse → render with no extras | `pulldown-cmark` |

## Performance

Release build, 50-iteration mean:

| Input size | Parse | Render | End-to-end |
|---|---|---|---|
| ~1 KB | 59 µs | 16 µs | 75 µs |
| ~10 KB | 643 µs | 156 µs | 799 µs |
| ~100 KB | 7.7 ms | 1.5 ms | 9.2 ms |

Suitable for interactive editors and real-time linting. On adversarial/pathological
input (deeply nested emphasis, unbalanced brackets) marco-core previously fell off
an algorithmic cliff (up to ~200x slower than comparable engines); as of this
release it stays within a small constant factor even on those inputs — see
[`PERFORMANCE_OPTIMIZATION_REVIEW.md`](PERFORMANCE_OPTIMIZATION_REVIEW.md) for
the full cross-engine comparison and methodology. Opt-in `parallel-render` and
`parallel-parse` features (off by default) add further multi-core speedups for
code-block-heavy and flat/wide documents — see [Feature flags](#feature-flags).

## Feature flags

8 flags are on by default. Use `default-features = false` to slim the build:

```toml
marco-core = { version = "1.3", default-features = false, features = ["render-syntax-highlighting"] }
```

| Flag | Enables |
|---|---|
| `render-math` | KaTeX rendering |
| `render-diagrams` | Mermaid rendering |
| `render-syntax-highlighting` | Code block syntax highlighting |
| `file-logger` | Log rotation and file logging |
| `intelligence-highlights` | Syntax highlight tags |
| `intelligence-diagnostics` | Linting and diagnostics |
| `intelligence-completions` | Autocompletion |
| `intelligence-hover` | Hover information |

A `--no-default-features` build still includes parse + basic render.

Two further flags are **off by default** (opt-in only, not part of the 8 above)
— both pull in `rayon` for a real OS thread pool, so they're excluded from
`default` for WASM/embedded targets that don't want threads:

| Flag | Enables |
|---|---|
| `parallel-render` | Fan out per-code-block syntax highlighting across cores at render time |
| `parallel-parse` | Fan out inline parsing of independent top-level blocks (paragraphs, table cells, definition terms, footnote bodies) across cores |

Both produce byte-for-byte/AST-identical output to the sequential path —
purely a performance opt-in. Call `warm_render_thread_pool(&["rust", "python"])`
at application startup to pre-pay `parallel-render`'s one-time thread-pool and
syntax-highlighter warm-up cost (a no-op when the feature isn't compiled in).

## Minimum Supported Rust Version

`marco-core` requires **Rust 1.94.1** (stable). The MSRV is pinned in CI and will not change without a minor version bump.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Detailed guides:
- [DEVELOPMENT.md](Documentation/DEVELOPMENT.md) — setup, workflow, coding rules
- [TESTING.md](Documentation/TESTING.md) — test inventory, CommonMark conformance
- [TOOLS.md](Documentation/TOOLS.md) — `marco-ast` CLI, `perf-lab` benchmarking

## License

MIT — see [LICENSE](LICENSE).

---

**Related:** [Marco](https://github.com/Ranrar/Marco) is the GTK4 Markdown editor built on this crate.
