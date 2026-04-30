# marco-core

[![Crates.io](https://img.shields.io/crates/v/marco-core.svg)](https://crates.io/crates/marco-core)
[![Docs.rs](https://img.shields.io/docsrs/marco-core)](https://docs.rs/marco-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 1.94.1](https://img.shields.io/badge/rust-1.94.1-orange.svg)](https://www.rust-lang.org)

`marco-core` is a pure-Rust Markdown engine: it sanitizes UTF-8 input, parses
Markdown into an AST with [nom][nom], renders that AST to HTML, and exposes
editor-facing "intelligence" (highlights, diagnostics, completions, hover) for
the [Marco][marco-editor] editor and any other downstream consumer.

It is a **library only** — no GUI, no binaries, no GTK/WebKit dependencies.
The single platform-gated dependency is `fontconfig` on Linux for font
discovery used by the math/diagram renderers.

[nom]: https://github.com/rust-bakery/nom
[marco-editor]: https://github.com/Ranrar/Marco

---

## Pipeline

```text
  raw input (&str)
        │
        ▼
  ┌──────────────────┐    logic::utf8
  │  sanitize_input  │    • UTF-8 validation (invalid → U+FFFD)
  │                  │    • Unicode NFC normalization
  │                  │    • line-ending + control-char filter
  └────────┬─────────┘
           │ clean &str
           ▼
  ┌──────────────────┐    grammar/  (nom combinators, Span-tracked)
  │ parse_blocks     │    blocks/   cm_* · gfm_* · marco_*
  │ parse_inlines    │    inlines/  cm_* · gfm_* · marco_* · math_*
  └────────┬─────────┘
           │
           ▼
  ┌──────────────────┐    parser::parse — three passes:
  │     Document     │     1. block + inline AST construction
  │ ┌──────────────┐ │     2. resolve reference-style links
  │ │ Vec<Node>    │ │     3. rewrite GFM alerts → Admonition
  │ │ ReferenceMap │ │
  │ └──────────────┘ │
  └────────┬─────────┘
           │ &Document
           ├──────────────────────────────┐
           ▼                              ▼
  ┌──────────────────┐           ┌──────────────────┐
  │  render::render  │           │   intelligence   │
  │ ──────────────── │           │ ──────────────── │
  │ markdown emitter │           │ highlights       │
  │ syntect codeblks │           │ diagnostics/lint │
  │ katex math       │           │ completion       │
  │ mermaid diagrams │           │ hover · toc      │
  └────────┬─────────┘           └────────┬─────────┘
           │                              │
           ▼                              ▼
        String                  Vec<Highlight>
        (HTML)                  Vec<Diagnostic>
                                Vec<CompletionItem>
                                Option<HoverInfo>
```

---

## Crate layout

```text
src/
├── lib.rs                      Re-exports the public surface
│
├── grammar/                    nom combinators → spans / tokens
│   ├── shared.rs               Span type, shared helpers
│   ├── blocks/
│   │   ├── cm_*.rs             CommonMark blocks
│   │   ├── gfm_table.rs        GFM pipe tables
│   │   └── marco_*.rs          Marco extensions (sliders, tabs, …)
│   └── inlines/
│       ├── cm_*.rs             CommonMark inlines
│       ├── gfm_strikethrough.rs
│       ├── math_inline.rs      $…$
│       ├── math_display.rs     $$…$$
│       └── marco_*.rs          Marco extensions
│
├── parser/                     Grammar output → AST
│   ├── ast.rs                  Document, Node, NodeKind, ReferenceMap
│   ├── position.rs             Position, Span
│   ├── blocks/                 Block AST builders
│   └── inlines/                Inline AST builders
│
├── render/                     AST → HTML
│   ├── markdown.rs             Core HTML emitter
│   ├── options.rs              RenderOptions
│   ├── syntect_highlighter.rs  Code-block highlighting
│   ├── math.rs                 KaTeX
│   ├── diagram.rs              Mermaid
│   ├── code_languages.rs       Language alias table
│   ├── plarform_mentions.rs    @user / @org rendering
│   ├── preview_document.rs     Standalone HTML doc wrapper
│   └── base_css.rs             Bundled CSS for previews
│
├── intelligence/               Editor-server features over the AST
│   ├── analysis/
│   │   ├── diagnostics.rs      Diagnostic, DiagnosticCode, severities
│   │   └── lint.rs             Lint reports / buckets
│   ├── editor/
│   │   ├── highlight.rs        Highlight, HighlightTag
│   │   ├── completion.rs       CompletionItem
│   │   └── hover.rs            HoverInfo
│   ├── catalog.rs              Diagnostic catalog loader
│   ├── diagnostics_catalog_*.ron
│   ├── lsp_protocol.rs         Optional LSP-shaped types
│   └── toc.rs                  Table-of-contents extraction
│
└── logic/                      Pure-Rust support
    ├── cache.rs                AST + HTML caches (moka)
    ├── utf8.rs                 sanitize_input, NFC, control filtering
    ├── text_completion.rs
    └── logger.rs               Optional file logger
```

Naming convention for parser-related modules:

| Prefix    | Meaning                            |
| --------- | ---------------------------------- |
| `cm_*`    | CommonMark spec feature            |
| `gfm_*`   | GitHub Flavored Markdown extension |
| `marco_*` | Marco-specific extension           |

---

## Supported Markdown

| Category           | Constructs                                                                                                                                                                                                                                                                                                              |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| CommonMark blocks  | ATX + setext headings, paragraphs, blockquotes, fenced & indented code, lists, link-reference defs, thematic breaks, HTML blocks                                                                                                                                                                                        |
| CommonMark inlines | Emphasis, strong, strong+emphasis, code spans, links (inline / reference / shortcut), images, autolinks, inline HTML, hard / soft breaks, backslash escapes, entity refs                                                                                                                                                |
| GFM                | Pipe tables (with alignment), strikethrough (`~~`), task lists, autolink literals, footnotes (`[^id]`), alerts / admonitions (`> [!NOTE]`)                                                                                                                                                                              |
| Math               | Inline `$…$` and display `$$…$$` (rendered via [katex-rs])                                                                                                                                                                                                                                                              |
| Diagrams           | Mermaid fenced blocks (rendered via [mermaid-rs-renderer])                                                                                                                                                                                                                                                              |
| Marco extensions   | Headerless tables, tab blocks (`:::tab` / `@tab`), sliders (`@slidestart` / `@slideend`), inline footnotes, mark (`==…==`), subscript / superscript, dash-strikethrough, emoji shortcodes (`:smile:`), platform mentions, extended definition lists, heading IDs (`{#id}`), inline task checkboxes |

[katex-rs]: https://crates.io/crates/katex-rs
[mermaid-rs-renderer]: https://crates.io/crates/mermaid-rs-renderer

---

## Public API

`src/lib.rs` re-exports the stable surface; everything else is internal and
may change without a major-version bump.

```rust
// Parsing
pub use parser::parse;                                    // &str -> Document
pub use parser::{Document, Node, NodeKind};

// Rendering
pub use render::{render, RenderOptions};                  // (&Document, &RenderOptions) -> String

// Intelligence
pub use intelligence::MarkdownIntelligenceProvider;

// Caching
pub use logic::cache::{parse_to_html, parse_to_html_cached, ParserCache};

// UTF-8 sanitization (call at the input boundary)
pub use logic::utf8::{
    sanitize_input, sanitize_input_with_stats,
    InputSource, SanitizeStats,
};
```

`RenderOptions`:

```rust
pub struct RenderOptions {
    pub syntax_highlighting: bool, // default: true
    pub line_numbers: bool,        // default: false
    pub theme: String,             // default: "github"
}
```

---

## Usage

Add the dependency:

```toml
[dependencies]
marco-core = "1.0"
```

### Parse + render

```rust
use marco_core::{parse, render, RenderOptions, sanitize_input, InputSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = "# Hello\n\n**café** — `code`";
    let clean = sanitize_input(raw, InputSource::File);

    let document = parse(&clean)?;
    let html = render(&document, &RenderOptions::default())?;

    println!("{html}");
    Ok(())
}
```

### Cached pipeline

`ParserCache` memoizes both the AST (keyed by content hash) and the rendered
HTML (keyed by content + render-options hash):

```rust
use marco_core::{parse_to_html_cached, ParserCache, RenderOptions};

let cache = ParserCache::new();
let opts  = RenderOptions::default();

let html1 = parse_to_html_cached("# hi", &opts, &cache)?; // miss → parse + render
let html2 = parse_to_html_cached("# hi", &opts, &cache)?; // hit  → cached HTML
assert_eq!(html1, html2);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Editor intelligence

```rust
use marco_core::{parse, MarkdownIntelligenceProvider};
use marco_core::parser::Position;

let source = "# Title\n\n[broken](javascript:alert(1))";
let document = parse(source)?;

let mut provider = MarkdownIntelligenceProvider::new();
provider.update_document(document);

let highlights  = provider.highlights(source);
let diagnostics = provider.diagnostics();
let completions = provider.completions("##");
let hover       = provider.hover(Position { line: 1, column: 3, offset: 2 });
# Ok::<(), Box<dyn std::error::Error>>(())
```

Diagnostics are emitted with stable codes such as `UnsafeLinkProtocol`,
`UnresolvedLinkReference`, `DuplicateHeadingId`, `EmptyCodeBlock`,
`InlineHtmlContainsScript`, etc. Codes are grouped by feature
(`MD1xx` headings, `MD2xx` links, `MD3xx` code, `MD4xx` images,
`MD5xx`/`MD6xx` HTML, `MD7xx` structural). See
[src/intelligence/analysis/diagnostics.rs](src/intelligence/analysis/diagnostics.rs)
and the `.ron` catalogs under
[src/intelligence/](src/intelligence/) for the full list.

---

## Building & testing

`marco-core` targets **stable Rust 1.94.1** (matching CI).

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked
cargo test  --locked
cargo doc   --open
cargo build --release
```

Tests:

- Unit / smoke tests live alongside their module (`#[cfg(test)] mod tests`).
- Integration tests live under [tests/](tests/), each exercising the public
  API (CommonMark gaps, GFM tables / footnotes / task lists / admonitions,
  Marco extensions, highlight computation, …).

CI runs on Linux (`fmt` + `clippy` + `cargo test`) and verifies a build on
Windows; releases are published to crates.io by
[.github/workflows/publish-crate.yml](.github/workflows/publish-crate.yml)
on tag push.

---

## Versioning

`marco-core` follows **independent SemVer** on the `1.x.y` track. The
re-exported surface in `lib.rs` is the API contract — additions go through a
minor bump, breaking changes through a major bump. See
[CHANGELOG.md](CHANGELOG.md) for the per-release diff.

---

## License

MIT — see [LICENSE](LICENSE).

## Related

- [Marco][marco-editor] — GTK4 Markdown editor that consumes this crate.
