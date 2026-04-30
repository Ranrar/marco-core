---
description: 'Rust instructions for marco-core (pure-Rust Markdown library)'
applyTo: '**/*.rs'
---

You are a Rust developer working on `marco-core`, a **pure-Rust library** crate published to crates.io. It provides a nom-based Markdown parser, an HTML renderer, and editor-intelligence features (highlights, diagnostics, completions, hover).

## Crate constraints

- **Library only** — no `main.rs`, no GUI, no GTK / SourceView / WebKit dependencies.
- **Single crate** — no workspace; do not introduce `marco`, `marco-shared`, or `polo` here.
- **Stable Rust 1.94.1** — matches CI; do not require nightly features.
- **API stability** — anything re-exported from `src/lib.rs` is the public contract. Treat additions as semver-minor and changes as semver-major.
- **Cross-platform** — keep code OS-agnostic; only `fontconfig` is gated behind `#[cfg(target_os = "linux")]`.

## Workflow

1. **Read before editing.** Inspect the relevant module (`grammar/`, `parser/`, `render/`, `intelligence/`, `logic/`) and any sibling files before changing behavior.
2. **Layered architecture.** Grammar produces spans/tokens, parser builds the AST, render emits HTML, intelligence consumes the AST. Do not skip layers or move logic across them.
3. **Make small, testable changes.** Add or update tests alongside the change.
4. **Verify locally**:
   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets --locked
   cargo test --locked
   ```
5. **Document user-visible changes** in `CHANGELOG.md` under `[Unreleased]` (Keep a Changelog format).

## Coding rules

- **No panics in library code.** Parser/render functions return `Result<T, Box<dyn std::error::Error>>` (or a more specific error). Avoid `unwrap()` / `expect()` outside tests; prefer `?`, `match`, or `.ok_or(...)`.
- **Borrow over clone.** Take `&str` / `&Document` in public APIs unless an owned value is required.
- **No `unsafe`** unless there is a documented, sound reason and tests covering it.
- **Naming**: use the `cm_*` (CommonMark), `gfm_*` (GitHub Flavored Markdown), `marco_*` (Marco extension) prefixes for new grammar/parser files.
- **OS gating**: use only `#[cfg(target_os = "linux")]` / `#[cfg(target_os = "windows")]`. Do not use `cfg(any(...))` / `cfg(not(...))` for OS gating.
- **Logging**: use the `log` crate (`log::debug!`, `log::info!`). Do not introduce `println!` / `eprintln!` in library code.

## Testing rules

- **Unit / smoke tests** live alongside the module (`#[cfg(test)] mod tests { ... }`).
- **Integration tests** live in `tests/*.rs` and exercise only the public API re-exported from `src/lib.rs`.
- A new grammar rule needs both: a unit test in the grammar module + an integration test in `tests/`.
- A bug fix needs a regression test that fails before the fix and passes after.
- The CommonMark spec suite lives in `tests/commonmark_spec_conformance.rs` and loads `tests/spec/*.json`. Bump `MIN_COMMONMARK_PASS` upward when conformance improves; never lower it without a documented reason. Set `MARCO_SPEC_VERBOSE=1` to print failures, `MARCO_SPEC_STRICT=1` to require 100% pass.

## When to refuse

- Requests that introduce a binary, GUI dependency, or cross-crate workspace structure: **decline** — those changes belong in the consuming editor (`Marco`).
- Requests that widen the public API without a clear use case: **push back** before adding `pub use` lines.
