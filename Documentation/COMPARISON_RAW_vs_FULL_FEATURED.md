# marco-core: Raw vs Full-Featured vs Other Parsers

Comparison of `marco-core` in two runtime modes against `pulldown-cmark` and `comrak`.

All measurements taken with a release build (`cargo build --release`), **50 iterations** per workload, via `perf-lab compare`. Timings are wall-clock nanoseconds (mean ± stdev).

---

## What "Raw" Means

| Mode | Track positions | Parse math | Parse diagrams |
|---|---|---|---|
| `marco-core` (full) | ✅ yes | ✅ yes | ✅ yes |
| `marco-core-raw` | ❌ no | ❌ no | ❌ no |

`marco-core-raw` calls `parse_with_options` with:

```rust
ParseOptions {
    track_positions: false,
    parse_math: false,
    parse_diagrams: false,
}
```

This removes the `LocatedSpan` construction (an O(n) UTF-8 scan per node) and skips the math / mermaid grammar branches entirely. No compile-time features are changed — the comparison is purely about **runtime options** when all features are compiled in.

---

## Parse Mode

Only parsing cost (no HTML rendering), 50 iterations.

| Workload | marco-core (ns) | marco-core-raw (ns) | raw gain | pulldown-cmark (ns) | comrak (ns) |
|---|---|---|---|---|---|
| small (~1 KB) | 6 472 | 5 512 | **+14%** | 740 | 1 428 |
| medium (~10 KB) | 146 263 | 131 724 | **+11%** | 12 410 | 21 647 |
| large (~100 KB) | 2 727 867 | 2 462 029 | **+11%** | 234 955 | 373 968 |
| pathological | 11 516 501 | 10 636 689 | **+8%** | 46 488 | 111 181 |
| spec:commonmark | 195 730 235 | 197 326 571 | ≈ 0% | 838 052 | 626 278 |
| spec:diagram | 463 405 | 433 073 | **+7%** | 27 160 | 61 252 |
| spec:gfm | 433 231 | 401 632 | **+8%** | 14 153 | 34 422 |
| spec:marco | 934 192 | 876 999 | **+6%** | 21 614 | 57 240 |
| spec:math | 261 858 | 280 420 | ≈ 0% | 15 498 | 37 067 |

### Speedup of pulldown-cmark / comrak vs marco-core (full) — Parse

| Workload | pulldown-cmark | comrak |
|---|---|---|
| small | 8.7× | 4.5× |
| medium | 11.8× | 6.8× |
| large | 11.6× | 7.3× |
| pathological | 247.7× | 103.6× |
| spec:commonmark | 233.6× | 312.5× |
| spec:gfm | 30.6× | 12.6× |
| spec:marco | 43.2× | 16.3× |

---

## End-to-End Mode (Parse + Render)

Full pipeline including HTML generation, 50 iterations.

| Workload | marco-core (ns) | marco-core-raw (ns) | raw gain | pulldown-cmark (ns) | comrak (ns) |
|---|---|---|---|---|---|
| small (~1 KB) | 8 246 | 7 607 | **+8%** | 1 004 | 1 857 |
| medium (~10 KB) | 178 697 | 170 660 | **+5%** | 15 176 | 27 437 |
| large (~100 KB) | 3 514 905 | 3 308 729 | **+6%** | 295 921 | 490 192 |
| pathological | 11 931 154 | 10 904 214 | **+9%** | 58 316 | 122 576 |
| spec:commonmark | 196 555 759 | 196 121 015 | ≈ 0% | 980 481 | 1 022 805 |
| spec:diagram | 528 342 | 503 621 | **+5%** | 30 309 | 66 326 |
| spec:gfm | 502 193 | 438 371 | **+13%** | 18 209 | 45 299 |
| spec:marco | 1 001 930 | 915 678 | **+9%** | 26 691 | 63 734 |
| spec:math | 306 591 | 308 649 | ≈ 0% | 18 568 | 42 369 |

### Speedup of pulldown-cmark / comrak vs marco-core (full) — E2E

| Workload | pulldown-cmark | comrak |
|---|---|---|
| small | 8.2× | 4.4× |
| medium | 11.8× | 6.5× |
| large | 11.9× | 7.2× |
| pathological | 204.6× | 97.3× |
| spec:commonmark | 200.5× | 192.2× |
| spec:gfm | 27.6× | 11.1× |
| spec:marco | 37.5× | 15.7× |

---

## Analysis

### Raw options: small but real gain

Disabling position tracking, math, and diagram parsing shaves **5–15%** off parse time on typical workloads. The saving is largest on GFM and Marco-extension content where the additional grammar branches see actual nodes. On math-heavy or diagram-heavy inputs (spec:math, spec:diagram) the gain is compressed because those parsers handle a small proportion of total work anyway.

On `spec:commonmark` the gain is ~0% — the CommonMark suite contains no math/diagrams and spans are allocated either way for the 652 CommonMark examples; position-skipping is not free on large corpora because the fallback still walks the input string.

The render step adds roughly the same overhead regardless of options because rendering is dominated by string allocation, not the parse-options flags.

### Why pulldown-cmark and comrak are faster

Both parsers trade feature breadth for throughput:

- **pulldown-cmark** is a streaming event iterator (zero-copy, no persistent AST). It cannot produce a navigable tree, source positions, diagnostics, or intelligence.
- **comrak** builds an AST but uses a tightly optimised arena allocator internally. It does not produce position spans by default and has no intelligence layer.

Neither parser supports: source positions, diagnostics, completions, hover, TOC, math rendering, diagram rendering, or syntax highlighting.

### marco-core scope

marco-core is a **feature-complete editor library**, not a throughput-maximising streaming tokenizer. The extra cost buys:

- Full `Position`/`Span` tracking (line/col ranges on every node — required for diagnostics and hover)
- GFM extensions (tables, strikethrough, task lists, footnotes, admonitions)
- Marco-specific extensions (tab blocks, sliders, inline footnotes, headerless tables, emoji shortcodes)
- Math rendering via KaTeX (`render-math` feature)
- Diagram rendering via Mermaid (`render-diagrams` feature)
- Syntax highlighting via syntect (`render-syntax-highlighting` feature)
- `MarkdownIntelligenceProvider`: diagnostics, completion, hover, and TOC

### When to use each mode

| Use case | Recommended |
|---|---|
| Editor / IDE integration (full features) | `marco_core::parse()` |
| Build-time HTML generation, positions not needed | `parse_with_options` with `track_positions: false` |
| Static-site batch conversion, standard CommonMark only | `pulldown-cmark` or `comrak` |
| Maximum throughput, no extensions | `pulldown-cmark` |

---

## Measurement Setup

```
Tool:       tools/perf-lab  (perf-lab v0.1.0)
Command:    perf-lab compare --engine <E1> ... --mode <mode> --iterations 50
Build:      cargo build --release --locked  (Rust stable 1.94.1)
Workloads:  synthetic small/medium/large/pathological + spec suites
```

Workload sizes (approximate):

| ID | Bytes |
|---|---|
| small | ~1 KB |
| medium | ~10 KB |
| large | ~100 KB |
| pathological | ~100 KB (deeply nested / repetitive) |
| spec:commonmark | ~600 KB (652 examples concatenated) |
| spec:gfm | ~80 KB |
| spec:marco | ~170 KB |
| spec:math | ~50 KB |
| spec:diagram | ~85 KB |

Raw JSON and CSV artifacts saved under `tools/perf-lab/output/summary/`.
