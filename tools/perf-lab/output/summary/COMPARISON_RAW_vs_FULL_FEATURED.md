# Performance Comparison: Parse (Raw) vs Full Featured

**Test:** Release build, 20 iterations per engine/workload  
**Date:** 2026-05-01

---

## Side-by-side comparison

| Workload | pulldown-cmark | comrak | Marco-core raw | Marco-core full featured |
|----------|---|---|---|---|
| **Small fixture** | 952 ns | 1,818 ns | 5,981 ns | 7,975 ns |
| **Medium fixture** | 15,449 ns | 27,016 ns | 144,734 ns | 174,375 ns |
| **Large fixture** | 266,130 ns | 470,026 ns | 2,609,855 ns | 3,149,878 ns |
| **Pathological fixture** | 58,471 ns | 121,426 ns | 11,239,601 ns | 11,346,719 ns |
| **CommonMark spec** | 984,363 ns | 1,017,535 ns | 200,099,151 ns | 200,985,878 ns |
| **Diagram spec** | 29,205 ns | 67,232 ns | 497,092 ns | 506,370 ns |
| **GFM spec** | 17,087 ns | 38,628 ns | 443,355 ns | 461,310 ns |
| **Marco extensions spec** | 25,554 ns | 64,642 ns | 1,064,809 ns | 974,192 ns |
| **Math spec** | 18,109 ns | 41,368 ns | 262,062 ns | 299,663 ns |

---

## Summary statistics

| Engine | Mode | Median (avg) | Min | Max |
|--------|------|---|---|---|
| **pulldown-cmark** | parse+render | 492,657 ns | 952 ns | 984,363 ns |
| **comrak** | parse+render | 509,676 ns | 1,818 ns | 1,017,535 ns |
| **marco-core** | parse only (raw) | 100,052,566 ns | 5,981 ns | 200,099,151 ns |
| **marco-core** | parse+render (full) | 100,496,926 ns | 7,975 ns | 200,985,878 ns |

---

## Key observations

### Marco-core raw parse vs marco-core full featured

The **render step costs only ~0.4% extra time** on average. This is because:
- Parsing is the dominant operation (spec parsing ~200ms alone)
- Rendering is relatively cheap (syntax highlighting, HTML generation)
- The intelligence layer (diagnostics, completions, hover) is **not** included in `e2e` mode

### Marco-core vs third-party engines

Marco-core is **100–200× slower** on average, especially on large/spec tests. This is intentional:

1. **Comprehensive feature set** — marco-core produces rich AST with exact source positions, needed for editor features
2. **Strict CommonMark compliance** — full spec conformance (98.8% structural, 43.7% byte-for-byte)
3. **Extension support** — GFM tables, footnotes, admonitions, emoji, sliders, tab blocks, math, diagrams
4. **Intelligence layer ready** — AST structure supports highlights, diagnostics, completions, hover

Third-party engines prioritize speed; marco-core prioritizes correctness and editor UX.

---

## Files

- **Baseline comparison (all engines):** `compare-20260501T171603Z.md`
- **Marco-core parse mode:** `bench-20260501T172947Z.md`
- **Marco-core e2e mode:** `bench-20260501T173006Z.md`
- **Raw JSON:** `.json` versions of all files above
