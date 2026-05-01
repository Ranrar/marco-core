# Contributing to marco-core

Thanks for your interest in `marco-core`. This document covers how to set up,
test, and submit changes.

## Crate scope

`marco-core` is a **standalone, pure-Rust library** published to crates.io.
It must remain:

- **Library only** — no `main.rs`, no binaries, no GUI dependencies (no GTK,
  SourceView, WebKit).
- **Self-contained** — no `marco`, `marco-shared`, or `polo` crates here.
- **API-stable** — anything re-exported from `src/lib.rs` follows semver.

If your change requires editor or UI code, open it against the consuming
editor at [Ranrar/Marco](https://github.com/Ranrar/Marco) instead.

## Development setup

```bash
git clone https://github.com/Ranrar/marco-core
cd marco-core
cargo build
cargo test --locked
```

Stable Rust **1.94.1** is the pinned toolchain (matches CI). On Linux the only
system package required is `libfontconfig-dev` (used by the math/diagram
renderers).

## Workflow

1. Open an issue for non-trivial changes so the design can be discussed first.
2. Fork the repo and create a topic branch.
3. Make small, focused commits.
4. Add or update tests covering the change (see "Testing" below).
5. Run the full check locally before pushing:
   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets --locked
   cargo test  --locked
   ```
6. Update `CHANGELOG.md` under `[Unreleased]` for any user-visible change
   (Keep a Changelog format: `Added` / `Changed` / `Fixed` / `Removed` /
   `Security`).
7. Open a pull request.

## Module layout

| Module           | Responsibility                                            |
| ---------------- | --------------------------------------------------------- |
| `grammar/`       | nom combinators that produce spans / tokens               |
| `parser/`        | AST builders that consume grammar output                  |
| `render/`        | AST → HTML emitter                                        |
| `intelligence/`  | Highlights, diagnostics, completions, hover, TOC          |
| `logic/`         | Pure-Rust support: cache, UTF-8 sanitize, logger          |

Do not skip layers. Use the file-name prefix convention for new
grammar/parser features:

- `cm_*` — CommonMark spec feature
- `gfm_*` — GitHub Flavored Markdown extension
- `marco_*` — Marco-specific extension

## Coding rules

- No panics in library code. Prefer `?`, `match`, or `.ok_or(...)` over
  `unwrap()` / `expect()` outside tests.
- Borrow over clone. Public APIs take `&str` / `&Document` unless ownership
  is required.
- No `unsafe` unless documented and tested.
- No `println!` / `eprintln!` in library code — use the `log` crate.
- OS gating uses only `#[cfg(target_os = "linux")]` /
  `#[cfg(target_os = "windows")]`. Do not use `cfg(any(...))` /
  `cfg(not(...))` for OS gating.

## Testing

- Unit / smoke tests live next to the module under
  `#[cfg(test)] mod tests { ... }`.
- Integration tests live in `tests/*.rs` and exercise only the public API
  re-exported from `src/lib.rs`.
- A new grammar rule needs **both** a unit test in the grammar module **and**
  an integration test under `tests/`.
- A bug fix needs a regression test that fails before the fix and passes
  after.
- Run the CommonMark spec suite locally:
  ```bash
  MARCO_SPEC_VERBOSE=1 cargo test --test commonmark_spec_it
  ```
  If your change improves conformance, bump `MIN_COMMONMARK_PASS` in
  `tests/commonmark_spec_it.rs` to the new measured baseline.

## Public API & semver

The contract is `src/lib.rs`. Adding a new `pub use` is a minor bump;
removing or changing one is a major bump. Discuss API additions in an issue
first — the public surface is intentionally small.

## Releasing

Maintainer-only:

1. Update `CHANGELOG.md` (`[Unreleased]` → versioned section).
2. Bump `version` in `Cargo.toml` (no leading zeros in any version part).
3. `cargo publish --dry-run --locked` to verify.
4. Commit, tag `vX.Y.Z`, and push the tag — the
   [`publish-crate.yml`](.github/workflows/publish-crate.yml) workflow
   handles crates.io publication.

## License

By contributing, you agree that your contributions will be licensed under
the [MIT License](LICENSE).
