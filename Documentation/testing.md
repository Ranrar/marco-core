# Testing in `marco-core`

A guided tour of every test in this repository: what it covers, how it
works, and how to read the numbers it prints.

> This is internal developer documentation. It is **not** shipped with the
> crate (`.dev/` is excluded from the published `.crate` tarball).

---

## 1. Test layout at a glance

```
src/**            — unit / smoke tests live next to the code (#[cfg(test)] mod tests)
tests/*.rs        — integration tests; one binary per file
tests/spec/*.json — fixtures consumed by the spec-conformance tests
```

| Layer        | Where           | What it tests                                   |
|--------------|-----------------|-------------------------------------------------|
| Unit         | `src/**`        | Internal helpers, parsers, renderers in isolation |
| Integration  | `tests/*.rs`    | Public API only (re-exports from `lib.rs`)      |
| Doc tests    | `///` examples  | Code in rustdoc comments                        |
| Spec         | `tests/spec/`   | CommonMark conformance fixtures                 |

Run them all:

```bash
cargo test --locked
```

Current total: **676 total (unit + integration + doc tests), all passing under `cargo test --locked`.**

---

## 2. The `tests/spec/` directory

Two JSON fixtures drive the conformance tests.

### `commonmark.json`

- **652 examples** from the official CommonMark 0.30 test suite
  (<https://spec.commonmark.org/>).
- Each entry has `markdown` (input), `html` (expected output),
  `example` (number), `section` (e.g. `"ATX headings"`),
  and source line range.
- Licensed **CC-BY-SA 4.0**. Attribution lives in
  [`tests/spec/README.md`](../tests/spec/README.md). It ships with the
  crate so downstream users can run conformance themselves.

### `extra.json`

- 4 Marco-specific extension cases that aren't in the spec.
- MIT-licensed like the rest of the crate.

---

## 3. The three "CommonMark numbers"

This is the part that has historically been confusing. There are **three
different ways** to ask "does our parser pass the CommonMark suite?", and
they give very different answers. All three are now visible in CI output.

### 3.1 Strict byte-for-byte equality — `commonmark_spec_it`

For every spec example:

```rust
let actual = render(&parse(&input)?, &RenderOptions::default())?;
if actual == expected_html { passed += 1; }
```

This is the **real** conformance number. It will catch:

- wrong content
- different attribute order
- different quote style (`"x"` vs `'x'`)
- extra/missing whitespace
- different entity escaping

**Current measured baseline: 285 / 652 ≈ 43.7 %.**

### 3.2 Loose structural match — `commonmark_spec_structural`

Mirrors the test runner the original Marco editor used. For each example
it builds a "signature" by checking which of these block-level tags
appear:

```
<h1 <h2 <h3 <h4 <h5 <h6 <p <pre <code <ul <ol <li <blockquote <hr <table
```

A case "passes" when the expected and actual signatures are identical
(including "neither has it"). It does **not** look at content,
attributes, or escaping.

**Current measured baseline: 644 / 652 ≈ 98.8 %.**

> **Why such a big gap (43.7 % vs 98.8 %)?**
> Most of the 367 strict failures are tiny formatting drift — the parser
> *did* produce a heading or paragraph in the right place, just not with
> byte-identical HTML. The structural test confirms the parser shape is
> correct in 98.8 % of cases; the strict test reveals how many of those
> are also formatted exactly the way the spec expects.

### 3.3 Historical Marco "100 % CommonMark Compliant" claim

The upstream Marco editor's `commonmark_tests.rs` checks only **5** tags
(`<h1>`, `<h2>`, `<p>`, `<code>`, `<pre>`) and counts "neither side has
it" as a pass. Under that very loose rule it reports **652 / 652 = 100 %**.

We do **not** replicate that test verbatim because it's misleading: a
case where the spec expects `<blockquote><p>x</p></blockquote>` and the
parser outputs `<p>wrong</p>` would "pass" because both contain `<p>`.

The new `commonmark_spec_structural` (Section 3.2) is the same idea done
honestly: 15 tags instead of 5, and exact signature equality.

### Summary table

| Test                          | Number       | What it actually checks                       |
|-------------------------------|--------------|-----------------------------------------------|
| `commonmark_spec_it` | 285 / 652    | Byte-for-byte HTML equality (real conformance)|
| `commonmark_spec_structural`  | 644 / 652    | Block-level tag set matches                   |
| `marco_extra_conformance`     | 4 / 4        | Marco extensions, strict                      |
| Marco editor's old runner     | 652 / 652    | Presence of any of 5 tags — informational only|

---

## 4. Regression-guard thresholds

The spec conformance tests are not asserting an exact number — they
assert a **floor**:

```rust
const MIN_COMMONMARK_PASS: usize = 280;  // measured baseline 285
const MIN_EXTRA_PASS:      usize = 4;    // measured baseline 4
```

```rust
// loose structural test
let min_structural_pct = 97.0;           // measured baseline 98.8
```

### Why a floor instead of `assert_eq!`

```
   pass count
        │
   652  ┤  (perfect — strict, future goal)
        │
   644  ┤  ← structural, today
        │
   285  ┤  ← strict, today
        │  ← any value in this band passes
   280  ┤  ← THRESHOLD: must stay ≥ this
        │
        ┤  ← below: test fails (regression!)
     0  ┤
```

- A small intentional formatting tweak that bumps strict from 285 → 288
  doesn't break the test (no need to remember to update a constant).
- A refactor that breaks 50 cases drops the count to 235 and **does**
  break the test. That's the whole point.
- When conformance improves materially, raise the constant. **Never
  lower it without a documented reason.**

### Strict mode

For audits or to push toward 100 % conformance:

```bash
MARCO_SPEC_STRICT=1   cargo test --test commonmark_spec_it
MARCO_SPEC_VERBOSE=1  cargo test --test commonmark_spec_it
```

- `MARCO_SPEC_STRICT=1` — assertion becomes "must pass 100 %".
- `MARCO_SPEC_VERBOSE=1` — prints up to 20 failing cases to stderr with
  expected / actual / markdown.

---

## 5. Integration test inventory (`tests/*.rs`)

Each file is one cargo test binary. Tests use only the public API
re-exported from `lib.rs`.

| File | Tests | Coverage |
|------|------:|---------|
| `autolink_highlighting_it.rs` | 1 | `compute_highlights` offsets for multi-byte URLs |
| `commonmark_features_it.rs` | 9 | Entity refs, ref-style links (full / collapsed / shortcut / unresolved / duplicate / Unicode casefold / escaped label) |
| `commonmark_spec_it.rs` | 3 | Strict byte-for-byte spec run + structural tag-set run + Marco extension suite |
| `definition_lists_it.rs` | 4 | `Term\n: Definition` AST structure, render to `<dl>/<dt>/<dd>`, nested blocks, non-matching lookalikes |
| `gfm_admonitions_it.rs` | 5 | `> [!NOTE]` callouts — all five standard kinds, custom header style, nested no-op, unknown marker no-op |
| `gfm_autolinks_it.rs` | 8 | Bare URL / `www.` / email autolinks, trailing punctuation trim, paren balancing, entity suffix trim |
| `gfm_footnotes_it.rs` | 3 | `[^1]` reference rendering, missing definition fallback, multiline definition body |
| `gfm_tables_it.rs` | 4 | Pipe-table parse, column alignment, row padding/truncation, setext-heading/table ambiguity |
| `gfm_tasklist_it.rs` | 6 | `- [x]` / `- [ ]` AST markers, SVG checkbox rendering, inline form, post-hardbreak form, link conflict |
| `heading_anchor_links_it.rs` | 2 | Custom `{#id}` anchor link, auto-slug fallback anchor |
| `heading_ids_it.rs` | 2 | `## Title {#custom-id}` parses `id` field, invalid syntax left in text |
| `highlighting_it.rs` | 4 | Multi-byte highlight offsets, tab-block markers, slider markers, source-matched vs AST-only |
| `html_autolink_it.rs` | 8 | Regression: `<img>`, `<span>`, `<div>` not parsed as autolinks; plain autolinks still work |
| `html_block_single_line_it.rs` | 1 | Single-line `<div>…</div>` does not swallow following Markdown |
| `intelligence_provider_it.rs` | 6 | `MarkdownIntelligenceProvider`: highlights, diagnostics (with severity filter), completions, hover (link + miss) |
| `marco_emoji_shortcode_it.rs` | 5 | `:joy:` mid-text parse, Unicode render, unknown shortcode literal, code-span protection, incomplete shortcode |
| `marco_headerless_table_it.rs` | 2 | `\|---\|---\|` first-row headerless table, does not break regular GFM tables |
| `marco_inline_footnotes_it.rs` | 4 | `^[inline note]` rendering, inline markup inside note, code-span protection, no superscript conflict |
| `marco_sliders_it.rs` | 3 | Slider deck AST build, HTML skeleton render, no nested deck creation |
| `marco_tab_blocks_it.rs` | 3 | Tab-group AST build, HTML skeleton render, no nested group creation |
| `parser_cache_it.rs` | 3 | `ParserCache` hit/miss, `parse_to_html`, `parse_to_html_cached` output parity |
| `platform_mentions_it.rs` | 3 | `@user[github]` default label, display override, unknown platform fallback `<span>` |
| `sanitize_input_it.rs` | 4 | Preserves clean UTF-8, strips null bytes, replaces invalid sequences, `SanitizeStats` counts |

**Total: 93 integration tests** (across 23 files). All pass under
`cargo test --locked`.

---

## 6. Conventions for new tests

### When to add what

| Change                          | Where to add a test                                       |
|---------------------------------|-----------------------------------------------------------|
| New grammar rule                | Smoke test in the grammar module + integration in `tests/`|
| New intelligence feature        | Integration test under `tests/`                           |
| HTML render change              | Integration test asserting exact HTML                     |
| Bug fix                         | Regression test that fails before the fix                 |

### Hard rules

1. Integration tests **must** use the public API from
   `marco_core::*` — not internal modules. The point is to catch
   accidental API breaks.
2. Tests must be deterministic and run in milliseconds. No sleeps, no
   network, no global mutable state.
3. Prefer real objects over mocks (this is a parser library — the
   "objects" are cheap).
4. New grammar rules need both a unit test next to the grammar file
   **and** an integration test that exercises parse → render.
5. Don't lower the spec conformance constants without a written reason.

### Naming convention

- `cm_*` — CommonMark spec features
- `gfm_*` — GitHub Flavored Markdown extensions
- `marco_*` — Marco-specific extensions
- `*_it.rs` — integration test binaries under `tests/`

---

## 7. Common commands

```bash
# Run everything
cargo test --locked

# Run one integration binary
cargo test --locked --test gfm_table_integration

# Run a single test by name
cargo test --locked some_test_name

# Verbose spec failures (first 20)
MARCO_SPEC_VERBOSE=1 cargo test --test commonmark_spec_it

# Demand 100% spec conformance (will fail today)
MARCO_SPEC_STRICT=1 cargo test --test commonmark_spec_it

# Format + lint + test (the full local check)
cargo fmt --all --check
cargo clippy --all-targets --locked
cargo test --locked
```

---

## 8. Known gaps and future work

- The strict-equality conformance number (285 / 652) is dragged down
  mostly by formatting drift, not real bugs. Closing that gap is a
  steady-grind task for the parser/renderer.
- Once strict ≥ 600, switch the regression guard from a floor to an
  exact match (`assert_eq!`) so silent improvements aren't lost.
- The structural test currently has 8 real shape mismatches — those
  are genuine parser bugs worth investigating before chasing the
  formatting-drift cases.
- A stale comment in [`src/parser/mod.rs`](../src/parser/mod.rs)
  claims "100 % CommonMark Compliant (652/652)". That number came from
  the loose 5-tag check described in §3.3 and contradicts the strict
  test. Worth correcting.
