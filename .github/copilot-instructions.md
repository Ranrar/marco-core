# marco-core Copilot Instructions

`marco-core` is a **standalone, pure-Rust library** crate published to crates.io. It provides a nom-based Markdown parser, an HTML renderer, and editor "intelligence" features (highlights, diagnostics, completions, hover) used by the [Marco](https://github.com/Ranrar/Marco) editor and other downstream tools.

This guide helps AI agents understand the crate's architecture and conventions.

## Communication Style

When completing work, **DO NOT create markdown documentation files**. Instead:
- Write summaries directly in chat responses
- Use simple tables for data
- Keep text blocks small and focused
- Be concise and to-the-point

## Problem-Solving Approach

1. **Review existing code** — check how similar things are handled elsewhere in the crate
2. **Search online** — find solutions, best practices, and nom/CommonMark documentation
3. **Analyze** — break down complex problems into smaller parts
4. **Test** — verify fixes with `cargo test` before considering work complete

## Crate Boundaries

`marco-core` is a **library only**. It must remain:

- **Pure Rust**, no GUI framework dependencies (no GTK, no SourceView, no WebKit)
- **Self-contained**, with no `marco`, `marco-shared`, or `polo` crates in the same repo
- **API-stable**, following semver for crates.io consumers

If a change requires editor/UI/binary code, it does **not** belong in this repo. Open the change against the consuming editor instead.

## Development Workflow

### Rust Toolchain

`marco-core` targets **stable Rust 1.94.1** (matching CI). Use:

```bash
cargo fmt --all              # Format
cargo clippy --all-targets   # Lint
cargo test --locked          # Run all tests (lib + integration)
cargo doc --open             # Build & open API docs
cargo build --release        # Release build
```

Optional:

```bash
cargo llvm-cov --html --open # Coverage report
```

### Testing Workflow

This is a library, so testing is the primary way to verify behavior:

1. Make the change
2. Run `cargo test --locked`
3. If parser/render output changes, update or add an integration test under `tests/`
4. Run `cargo clippy --all-targets --locked` and `cargo fmt --all --check`

There is no application binary, no log file, and no runtime UI to inspect.

## Architecture Overview

Single crate `marco-core` with the following module layout under `src/`:

| Module | Purpose |
|---|---|
| `grammar/` | nom-based grammar parsers for block + inline Markdown elements |
| `parser/` | AST builders that consume grammar output (`ast.rs`, `position.rs`, `blocks/`, `inlines/`) |
| `render/` | HTML renderer (entity escaping, syntect highlighting, KaTeX, Mermaid) |
| `intelligence/` | Editor features: highlights, diagnostics, completions, hover, TOC |
| `logic/` | Pure Rust support: cache, logger, UTF-8 sanitization, text completion |

### Parser Pipeline

```rust
// Core workflow: grammar → parser → AST → renderer
let document = marco_core::parse(input)?;
let html = marco_core::render(&document, &marco_core::RenderOptions::default())?;
```

Key files:

- `src/grammar/blocks/*.rs` — block-level grammars (headings, code blocks, lists, tables, …)
- `src/grammar/inlines/*.rs` — inline grammars (emphasis, links, autolinks, math, …)
- `src/parser/blocks/*.rs`, `src/parser/inlines/*.rs` — AST builders calling grammar functions
- `src/parser/ast.rs` — `Document`, `Node`, `NodeKind`, `Position`, `Span`
- `src/render/markdown.rs` — HTML output with entity escaping

Naming convention for grammar/parser files:

- `cm_*` — CommonMark spec features
- `gfm_*` — GitHub Flavored Markdown extensions
- `marco_*` — Marco-specific extensions

### Intelligence Layer (`src/intelligence/`)

Editor-server-style features computed from the AST:

- `analysis/highlights*` — syntax highlight tags
- `analysis/diagnostics*` — parse validation with severities (Error, Warning, Info, Hint)
- `analysis/completion*` — context-aware suggestions
- `analysis/hover*` — hover information
- `catalog.rs` + `diagnostics_catalog_*.ron` — diagnostic message catalog
- `lsp_protocol.rs` — LSP-shaped types
- `toc.rs` — table-of-contents extraction

Public API entry point: `MarkdownIntelligenceProvider` (re-exported from `lib.rs`).

### Public API

`src/lib.rs` re-exports the stable surface:

```rust
pub use parser::parse;
pub use parser::{Document, Node, NodeKind};
pub use render::{render, RenderOptions};
pub use intelligence::MarkdownIntelligenceProvider;
pub use logic::cache::{parse_to_html, parse_to_html_cached, ParserCache};
pub use logic::utf8::{sanitize_input, sanitize_input_with_stats, InputSource, SanitizeStats};
```

Anything not re-exported from `lib.rs` is internal and may change without a major version bump.

## Code Organization Rules

1. **Library only** — no `main.rs`, no binaries, no GUI deps
2. **Module discipline** — grammar produces tokens/spans, parser builds the AST, render emits HTML, intelligence consumes the AST. Do not skip layers.
3. **Naming** — keep `cm_` / `gfm_` / `marco_` prefixes consistent for new grammar/parser features
4. **Public API** — only widen `lib.rs` re-exports deliberately; treat additions as semver-relevant
5. **Errors** — parser/render functions return `Result<T, Box<dyn std::error::Error>>` (or a more specific error if introduced); avoid panics in library code
6. **Cross-platform** — keep code OS-agnostic. The only platform-gated dependency today is `fontconfig` on Linux for font discovery; gate any future OS-specific code with `#[cfg(target_os = "linux")]` / `#[cfg(target_os = "windows")]` (do not use `cfg(any(...))` / `cfg(not(...))` for OS gating)

## Versioning & Release

`marco-core` follows **independent semver** on the `1.x.y` track and is published to crates.io.

- Source of truth for the version: `Cargo.toml` (single crate, no workspace)
- Update `CHANGELOG.md` for every user-visible change (Keep a Changelog format: `Added`, `Changed`, `Fixed`, `Removed`, `Security`)
- Breaking changes to the `lib.rs` re-exported API → major bump
- Tag releases as `vX.Y.Z` and let `.github/workflows/publish-crate.yml` handle crates.io publication

### SemVer Zero-Padding Policy

- No leading zeros in `major.minor.patch` (`1.2.3` ✅, `01.2.3` ❌)
- A single `0` digit is fine
- Pre-release / build metadata allowed (`1.0.0-rc.1`, `2.0.0+build.123`) but their numeric parts also must not have leading zeros

## CI Workflows

Workflows live in `.github/workflows/`:

- `ci-linux.yml` — fmt, clippy, tests on Linux (Rust 1.94.1)
- `ci-windows.yml` — build verification on Windows
- `publish-crate.yml` — publishes to crates.io on tagged releases
- `devskim.yml` — security scanning

CI must build deterministically — never mutate `Cargo.toml` versions during a CI run.

## Testing Approach

### Primary: Smoke + Integration Tests

- **Unit / smoke tests** live alongside their module (`#[cfg(test)] mod tests { … }`)
- **Integration tests** live in `tests/*.rs`, each testing a public scenario (CommonMark gaps, GFM tables, footnotes, highlighting, Marco extensions, …)

Smoke tests should:
- Run in milliseconds
- Use the real public API (no mocking)
- Have clear, observable assertions
- Be self-contained

### When to Add Tests

- New grammar rule → smoke test under the grammar module + integration test under `tests/`
- New intelligence feature → smoke test for highlights/diagnostics/completion as appropriate
- HTML render change → integration test asserting the exact HTML string
- Bug fix → regression test that fails before the fix and passes after

### Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parser_cache() {
        let cache = SimpleParserCache::new();
        let content = "# Hello World\n\nThis is a **test** document.";

        let _ast1 = cache.parse_with_cache(content).expect("parse failed");
        let stats = cache.stats();
        assert_eq!(stats.ast_misses, 1);
        assert_eq!(stats.ast_hits, 0);

        let _ast2 = cache.parse_with_cache(content).expect("parse failed");
        let stats = cache.stats();
        assert_eq!(stats.ast_hits, 1);
    }
}
```

### Guidelines

1. Smoke tests first — every new module ships with tests
2. Test the public API surface from `lib.rs` whenever possible
3. Prefer real objects over mocks
4. Keep tests fast — they run on every push
5. Run `cargo test --locked` before marking work complete
