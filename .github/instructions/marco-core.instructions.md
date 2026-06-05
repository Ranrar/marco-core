---
description: 'Rust instructions for marco-core (pure-Rust Markdown library)'
applyTo: '**/*.rs'
---

Apply Rust library development expertise to `marco-core`, a **pure-Rust library** crate (v1.1.1) published to crates.io — a nom-based Markdown parser, HTML renderer, and editor-intelligence layer (highlights, diagnostics, completions, hover).

## Crate constraints

- **Library only** — no `main.rs`, no GUI, no GTK / SourceView / WebKit dependencies.
- **Single crate** — no workspace; do not introduce `marco`, `marco-shared`, or `polo` here.
- **Stable Rust 1.94.1** — matches `rust-version` in `Cargo.toml` and CI; do not require nightly features.
- **API stability** — anything re-exported from `src/lib.rs` is the public contract. Treat additions as semver-minor and changes as semver-major.
- **Cross-platform** — keep code OS-agnostic; only `fontconfig` is gated behind `#[cfg(target_os = "linux")]`.

## Module layout

```
src/
  lib.rs                          — public API surface (re-exports only)
  grammar/
    shared.rs                     — shared nom combinators
    blocks/                       — cm_*, gfm_*, marco_* block grammars
    inlines/                      — cm_*, gfm_*, marco_*, math_* inline grammars
  parser/
    ast.rs                        — Document, Node, NodeKind, Position, Span
    position.rs / shared.rs
    blocks/                       — one *_parser.rs per block grammar file
    inlines/                      — one *_parser.rs per inline grammar file
  render/
    markdown.rs                   — main HTML renderer
    options.rs                    — RenderOptions
    base_css.rs                   — bundled CSS (marco-* / mc-* class namespace)
    syntect_highlighter.rs        — syntax highlighting via syntect
    diagram.rs / math.rs          — Mermaid and KaTeX support
    preview_document.rs           — full HTML document wrapper
    plarform_mentions.rs          — @mention rendering
    code_languages.rs             — language alias table
  intelligence/
    mod.rs / lsp_protocol.rs / catalog.rs
    analysis/diagnostics.rs / lint.rs
    editor/completion.rs / highlight.rs / hover.rs
    markdown/ast.rs / blocks.rs / inlines.rs
    toc.rs
  logic/
    logger.rs / utf8.rs / text_completion.rs
```

**Tools (not part of the library crate):**
- `tools/perf-lab/` — standalone benchmark crate (`publish = false`); adapters for marco-core, pulldown-cmark, comrak
- `tools/marco-ast/` — standalone AST introspection CLI (`publish = false`)
- `tools/spec/` — JSON spec fixtures: `commonmark.json`, `gfm.json`, `marco.json`, `math.json`, `diagram.json`
- `tools/tests/` — `extension_spec_it.rs` integration test for GFM/marco/math/diagram specs

## Workflow

1. **Read before editing.** Inspect the relevant module (`grammar/`, `parser/`, `render/`, `intelligence/`, `logic/`) and any sibling files before changing behavior.
2. **Layered architecture.** Grammar produces spans/tokens → parser builds the AST → render emits HTML → intelligence consumes the AST. Do not skip layers or move logic across them.
3. **Make small, testable changes.** Add or update tests alongside the change.
4. **Verify locally**:
   ```bash
   cargo fmt --all                  # auto-format all files
   cargo fmt --all --check          # CI-style: fail if formatting differs
   cargo clippy --all-targets --locked
   cargo test --locked
   ```
5. **Document user-visible changes** in `CHANGELOG.md` under `[Unreleased]` (Keep a Changelog format).

## Coding rules

- **No panics in library code.** Parser/render functions return `Result<T, Box<dyn std::error::Error>>` (or a more specific error). Avoid `unwrap()` / `expect()` outside tests; prefer `?`, `match`, or `.ok_or(...)`.
- **Borrow over clone.** Take `&str` / `&Document` in public APIs unless an owned value is required.
- **No `unsafe`** unless there is a documented, sound reason and tests covering it.
- **Naming**: use the `cm_*` (CommonMark), `gfm_*` (GitHub Flavored Markdown), `marco_*` (Marco extension) prefixes for new grammar/parser files. Math and diagram grammars use `math_*` / `diagram_*`.
- **CSS classes**: all HTML output uses the `marco-*` prefix for block/inline classes and `mc-*` for custom properties. Do not introduce unprefixed classes.
- **OS gating**: use only `#[cfg(target_os = "linux")]` / `#[cfg(target_os = "windows")]`. Do not use `cfg(any(...))` / `cfg(not(...))` for OS gating.
- **Logging**: use the `log` crate (`log::debug!`, `log::info!`). Do not introduce `println!` / `eprintln!` in library code.
- **Key dependencies**: `nom 8` + `nom_locate 5` for parsing; `syntect 5` for highlighting; `serde`/`serde_json` for config; `log 0.4`; `chrono 0.4` for timestamps; `fontconfig 0.10` (Linux only).

## Testing rules

- **Unit / smoke tests** live alongside the module (`#[cfg(test)] mod tests { ... }`).
- **Integration tests** live in `tests/*_it.rs` (note the `_it` suffix) and exercise only the public API re-exported from `src/lib.rs`.
- `tools/tests/extension_spec_it.rs` exercises GFM, Marco, math, and diagram spec fixtures from `tools/spec/`.
- A new grammar rule needs both: a unit test in the grammar module + an integration test in `tests/`.
- A bug fix needs a regression test that fails before the fix and passes after.
- The CommonMark spec suite is `tests/commonmark_spec_it.rs`; it loads `tools/spec/commonmark.json` and gates on `MIN_COMMONMARK_PASS` (currently 280). Bump the constant upward when conformance improves; never lower it without a documented reason.
  - `MARCO_SPEC_VERBOSE=1` — print failing examples to stderr
  - `MARCO_SPEC_STRICT=1` — require 100% pass

## Current integration tests (`tests/*_it.rs`)

| File | Coverage area |
|---|---|
| `autolink_highlighting_it.rs` | autolink syntax highlights |
| `commonmark_features_it.rs` | CommonMark block/inline features |
| `commonmark_spec_it.rs` | Full CommonMark spec conformance |
| `definition_lists_it.rs` | GFM definition lists |
| `gfm_admonitions_it.rs` | GFM admonition blocks |
| `gfm_autolinks_it.rs` | GFM autolink literals |
| `gfm_footnotes_it.rs` | GFM footnotes |
| `gfm_tables_it.rs` | GFM tables |
| `gfm_tasklist_it.rs` | GFM task list items |
| `heading_anchor_links_it.rs` | Heading anchor link generation |
| `heading_ids_it.rs` | Extended heading ID attributes |
| `highlighting_it.rs` | Syntax highlight tags |
| `html_autolink_it.rs` | HTML autolink rendering |
| `html_block_single_line_it.rs` | Single-line HTML blocks |
| `intelligence_provider_it.rs` | `MarkdownIntelligenceProvider` |
| `marco_emoji_shortcode_it.rs` | Marco emoji shortcodes |
| `marco_headerless_table_it.rs` | Marco headerless tables |
| `marco_inline_footnotes_it.rs` | Marco inline footnotes |
| `marco_sliders_it.rs` | Marco slider deck blocks |
| `marco_tab_blocks_it.rs` | Marco tab blocks |
| `platform_mentions_it.rs` | @mention rendering |
| `sanitize_input_it.rs` | UTF-8 sanitization |

## When to refuse

- Requests that introduce a binary, GUI dependency, or cross-crate workspace structure: **decline** — those changes belong in the consuming editor (`Marco`).
- Requests that widen the public API without a clear use case: **push back** before adding `pub use` lines.
