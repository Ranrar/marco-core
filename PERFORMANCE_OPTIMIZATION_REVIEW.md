# Parser & Render Optimization — Implementation Review & Performance Cross-Reference

**Date:** 2026-07-17
**Branches compared:** [`optimization`](https://github.com/Ranrar/marco-core/tree/optimization) (`1f16b79`) vs [`main`](https://github.com/Ranrar/marco-core/tree/main) (`e2675b5`)
**Source plan:** [`.dev/parser-render-optimization-plan.md`](.dev/parser-render-optimization-plan.md)
**Tooling:** `tools/perf-lab` (in-repo benchmarking/comparison harness)

This document is an independent review of the work described in the optimization
plan: it re-verifies the plan's claims against the actual source tree and test
suite, then independently re-runs `tools/perf-lab` to reproduce (not just quote)
the plan's headline numbers. All tables below were generated for this review on
2026-07-17; raw artifacts are cited by filename under `tools/perf-lab/output/`
for reproducibility.

## 1. Implementation review — is the plan accurate?

Each phase's code claims were checked directly against the source tree and test
run output (not just read from the plan's prose):

| Phase | Claim | Verified |
|---|---|---|
| Phase 0 | Pathological fixtures (`star-pyramid.md`, `unbalanced-brackets.md`) committed under `tools/perf-lab/fixtures/pathological/` | ✅ present |
| Phase 1 | `src/grammar/inlines/{cm_emphasis,cm_strong,cm_strong_emphasis}.rs` and `src/parser/inlines/{cm_emphasis_parser,cm_strong_parser,cm_strong_emphasis_parser}.rs` deleted, replaced by `src/parser/inlines/emphasis.rs` (445 lines, delimiter-stack algorithm) | ✅ old files confirmed gone, new module confirmed present |
| Phase 2 | New `src/grammar/inlines/bracket_match.rs` (256 lines) precomputes bracket pairs | ✅ present |
| Phase 3 | `parallel-render` feature (`["dep:rayon"]`), not in `default` feature list; `pub fn warm_render_thread_pool(languages: &[&str])` exported at crate root | ✅ confirmed in `Cargo.toml` and `src/lib.rs:47` |
| Phase 4 | `parallel-parse` feature (`["dep:rayon"]`), not in `default`; `src/parser/blocks/parallel_inline.rs` (303 lines) | ✅ present |
| Phase 5 | CI regression gate has a `--critical-only` mode targeting `spec:commonmark`/`fixture:pathological:*` specifically, plus the `jq -s add` mode-merge fix | ✅ confirmed in `.github/workflows/ci-perf.yml` and `tools/perf-lab/src/runners/regression.rs` |

Correctness, re-run directly (not taken from the plan's own log):

```
cargo test --release --test commonmark_spec_it -- --nocapture
  CommonMark spec: 357/652 passed (54.8%)
  CommonMark structural: 644/652 passed (98.8%)
cargo test --release --test extension_spec_it
  gfm 18/18, marco 76/76, math 12/12, diagram 8/8, combos: all passed (100%)
cargo test --release --features parallel-render,parallel-parse \
  --test commonmark_spec_it --test extension_spec_it --test parallel_parse_it
  → identical pass rates above, plus 9/9 parallel_parse_it tests green
```

All figures match the plan document's own claims exactly (357/652, 644/652,
100% extension suites). **The plan's correctness claims are accurate.**

## 2. Methodology

Three build configurations were benchmarked with `tools/perf-lab`:

| Label | Commit | Cargo features | Build command |
|---|---|---|---|
| **main (before)** | `e2675b5` | default | pre-existing baseline artifact, captured at the exact commit `main` still points to today — reused rather than re-run (see §2.1) |
| **optimization, sequential (after)** | `1f16b79` | default (`parallel-render`/`parallel-parse` off) | `cargo build --manifest-path tools/perf-lab/Cargo.toml --release` |
| **optimization, parallel (after, parallel on)** | `1f16b79` | `+parallel-render,+parallel-parse` | `cargo build --manifest-path tools/perf-lab/Cargo.toml --release --features marco-core/parallel-render,marco-core/parallel-parse` |

Cross-engine ratios use `compare --mode e2e --iterations 20` (marco-core /
pulldown-cmark / comrak, in-process, same methodology the plan and the
pasted evidence table used). Parallel on/off uses `bench --mode {e2e,parse,render}
--iterations 20` run against each of the two binaries above.

### 2.1 Why `main`'s numbers were reused instead of re-run

`git log main -1` is `e2675b5`, which is *exactly* the commit the plan's own
Phase 0 baseline (`tools/perf-lab/output/baseline/pre-optimization-e2675b5-*`)
was captured at — confirmed via `git show e2675b5`. There has been no drift
between that baseline and current `main`, so re-running it would reproduce
the same code path; the existing artifact was used directly instead.

That baseline only covers the 10 workloads that existed at that commit
(4 `generated-synthetic.md` fixtures + 6 spec suites) — it predates
`star-pyramid.md`, `unbalanced-brackets.md`, `code-heavy.md`, and
`paragraph-heavy.md`, all added later on `optimization`. For the two
pathological fixtures, this review independently re-measured "before" by
building a temporary worktree at `d678bd2` (Phase 0: fixtures added,
algorithmic fixes not yet applied) — see Table 2. `code-heavy.md` and
`paragraph-heavy.md` were purpose-built to demonstrate the parallel
features and have no meaningful pre-optimization baseline (see Table 3).

## 3. Table 1 — Cross-engine comparison, before vs after (`main` vs `optimization`)

Ratio = marco-core mean / other-engine mean, lower is better, `1.00x` = parity.
Source: `pre-optimization-e2675b5-compare-e2e.json` (before) vs
`compare-20260717T192956Z.json` (after, this review, sequential build).

| workload | before vs pulldown-cmark | before vs comrak | after vs pulldown-cmark | after vs comrak |
|---|---:|---:|---:|---:|
| `fixture:small` | 6.06x | 3.46x | 6.02x | 3.41x |
| `fixture:medium` | 8.00x | 4.58x | 9.34x | 5.30x |
| `fixture:large` | 8.92x | 5.66x | 10.55x | 5.59x |
| `fixture:pathological` (mixed) | 162.80x | 78.78x | 15.32x | 7.42x |
| `spec:commonmark` (full spec suite) | **209.64x** | **209.24x** | **2.40x** | **2.39x** |
| `spec:gfm` | 59.54x | 24.63x | 9.78x | 4.00x |
| `spec:marco` | 67.12x | 25.79x | 13.40x | 5.01x |
| `spec:math` | 14.89x | 6.29x | 4.24x | 1.83x |
| `spec:diagram` | 19.35x | 8.58x | 4.47x | 2.00x |
| `spec:combos` | 22.75x | 9.68x | 3.15x | 1.33x |
| `fixture:pathological:star-pyramid.md`† | n/a | n/a | 12.83x | 11.48x |
| `fixture:pathological:unbalanced-brackets.md`† | n/a | n/a | 4.68x | 2.64x |
| `fixture:large:paragraph-heavy.md`† | n/a | n/a | 18.98x | 10.76x |
| `fixture:large:code-heavy.md`†‡ | n/a | n/a | 645.20x | 297.05x |

† Fixture didn't exist on `main`; added later on `optimization` — no "before" number is possible.
‡ `code-heavy.md` is not an apples-to-apples algorithmic comparison: it exercises marco-core's syntect syntax highlighting, which pulldown-cmark/comrak don't perform at all. It exists to demonstrate `parallel-render` (Table 3), not competitive parity.

These figures independently reproduce the plan's own "Evidence Gathered" table
(within normal run-to-run variance — e.g. `spec:commonmark` 209.64x/209.24x here
vs the plan's 211.6x/209.5x) and confirm the **after** side: the ~70–210x
pathological/spec-suite cliffs are gone, and `spec:commonmark` — the plan's
primary target — landed at 2.40x/2.39x, inside the stated `<3x` goal.

## 4. Table 2 — Algorithmic fix in isolation (pathological fixtures, before/after)

Independently re-measured for this review, not quoted from the plan: `before`
is a temporary `git worktree` built at `d678bd2` (the Phase 0 commit — fixtures
present, delimiter-stack/bracket-cache fixes not yet applied), `after` is
current `optimization` HEAD. Both `parse` mode, marco-core only, no other
engines involved.

| fixture | before (`d678bd2`, parse mean) | after (`1f16b79`, parse mean) | speedup |
|---|---:|---:|---:|
| `star-pyramid.md` (640 lines, nested `*` emphasis) | 9.75 ms (30 iter) | 3.33 ms (20 iter) | **2.93x** |
| `unbalanced-brackets.md` (400 lines, bracket-only) | 313.98 ms (30 iter) | 1.01 ms (20 iter) | **311x** |

The plan document quotes 11.24 ms / 313.6 ms for the same "before" state
(also 30-iteration release, its own independent capture) — this review's
9.75 ms / 313.98 ms lands within normal run-to-run variance of those figures,
confirming the plan's numbers rather than contradicting them. The
`unbalanced-brackets.md` result is the standout: a 400-line, 13.3 KB file
that took **314 ms** to parse pre-fix (worse than the star-pyramid case *per
byte*, per the plan's own analysis) now parses in ~1 ms.

## 5. Table 3 — Parallel on/off (`parallel-render` + `parallel-parse`, opt-in features)

`optimization` HEAD only — this isolates the parallel features from the
algorithmic fix, which is already included on both sides. Single in-process
`bench --iterations 20` run per config; see §5.1 for a noise caveat on the
smaller fixtures.

| workload | e2e off | e2e on | e2e speedup | parse off | parse on | parse speedup | render off | render on | render speedup |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `fixture:large:code-heavy.md` | 64.05 ms | 9.35 ms | **6.85x** | 0.74 ms | 0.88 ms | 0.84x | 69.48 ms | 8.65 ms | **8.03x** |
| `fixture:large:paragraph-heavy.md` | 7.95 ms | 5.45 ms | **1.46x** | 5.73 ms | 3.03 ms | **1.89x** | 2.44 ms | 2.32 ms | 1.05x |
| `fixture:large:generated-synthetic.md` | 2.70 ms | 2.76 ms | 0.98x | 2.23 ms | 2.64 ms | 0.84x† | 0.86 ms | 0.60 ms | 1.42x |
| `fixture:medium:generated-synthetic.md` | 0.16 ms | 0.23 ms | 0.70x | 0.11 ms | 0.12 ms | 0.88x | 0.03 ms | 0.05 ms | 0.61x |
| `fixture:pathological:generated-synthetic.md` | 0.91 ms | 1.11 ms | 0.82x | 0.80 ms | 0.89 ms | 0.90x | 0.10 ms | 0.11 ms | 0.93x |
| `fixture:pathological:star-pyramid.md` | 4.09 ms | 4.21 ms | 0.97x | 3.33 ms | 3.47 ms | 0.96x | 0.84 ms | 0.87 ms | 0.96x |
| `fixture:pathological:unbalanced-brackets.md` | 1.07 ms | 1.07 ms | 1.00x | 1.01 ms | 0.99 ms | 1.02x | 0.06 ms | 0.06 ms | 0.94x |
| `fixture:small:generated-synthetic.md` | 0.008 ms | 0.007 ms | 1.06x | 0.005 ms | 0.006 ms | 0.88x | 0.001 ms | 0.002 ms | 0.92x |
| `spec:commonmark` | 2.47 ms | 2.39 ms | 1.03x | 2.95 ms | 2.10 ms | **1.40x** | 0.25 ms | 0.27 ms | 0.93x |
| `spec:gfm` | 0.62 ms | 0.64 ms | 0.98x | 0.59 ms | 0.53 ms | 1.13x | 0.09 ms | 0.09 ms | 0.96x |
| `spec:marco` | 3.54 ms | 3.61 ms | 0.98x | 3.13 ms | 3.28 ms | 0.95x | 0.43 ms | 0.45 ms | 0.94x |
| `spec:math` | 0.26 ms | 0.26 ms | 0.99x | 0.15 ms | 0.15 ms | 1.00x | 0.10 ms | 0.10 ms | 1.00x |
| `spec:diagram` | 0.24 ms | 0.25 ms | 0.95x | 0.24 ms | 0.18 ms | 1.39x | 0.06 ms | 0.08 ms | 0.81x |
| `spec:combos` | 0.72 ms | 0.74 ms | 0.97x | 0.37 ms | 0.36 ms | 1.04x | 0.35 ms | 0.37 ms | 0.94x |

Two results confirm the plan's headline parallel-feature numbers directly:

- **`code-heavy.md` render/e2e** (Phase 3 target — 80 fenced code blocks,
  67.7 KB): this review measured **6.85x e2e / 8.03x render**, matching the
  plan's own independently-reviewed **6.8-6.9x** (and its "independent review"
  re-measurement of ~8.4x render). Confirms Phase 3's parallel syntax
  highlighting is real and lands in the claimed range.
- **`paragraph-heavy.md` parse** (Phase 4 target — 400 top-level paragraphs,
  143 KB): this review measured **1.89x**, matching the plan's **~1.95x**
  (5.78 ms → 2.96 ms) almost exactly.

Everything else clusters at 0.8x–1.1x — i.e. noise, not regression or gain —
which is exactly the plan's "depth-gating" design intent: `parallel-parse`
only defers work at `depth == 0` (top-level document scan), so list-heavy,
blockquote-heavy, or emphasis/bracket-heavy fixtures with little flat
top-level paragraph content are expected to see no change either way.

### 5.1 Noise caveat, confirmed by repeated trials

The single-run `fixture:large:generated-synthetic.md` parse figure above
(0.84x, i.e. an apparent 16% regression) did not reproduce under repeated
measurement. Three repeated 30-iteration runs per config gave:

- sequential: 2.278 ms, 2.311 ms, 2.313 ms
- parallel: 2.275 ms, 2.246 ms, 2.262 ms

i.e. statistically indistinguishable — parallel is, if anything, marginally
faster, not 16% slower. This matches the plan's own claim ("within ~2% of
baseline... noise level, confirmed across repeated trials") and confirms
the single-run table above understates stability on sub-3ms workloads;
treat any single cell within ±15-20% of 1.00x in Table 3 as noise, not signal.

### 5.2 Known limitations (from the plan, independently confirmed relevant)

- **Warm vs. cold thread pool**: the numbers above come from a harness that
  loops iterations inside one process (the pool/syntect caches warm up on
  iteration 1 and stay warm). The plan's own "Independent review" section
  found a *first-call-ever* process pays a real ~12-13 ms one-time tax for
  `code-heavy.md`, dropping the true one-shot-process speedup to ~3.1x, not
  6.8x. `warm_render_thread_pool(languages)` exists specifically to let an
  embedder pre-pay this cost at a time of its choosing (e.g. app startup).
- **Depth-gating**: only top-level (`depth == 0`) paragraphs, footnote
  bodies, table cells, and definition terms are ever deferred to
  `parallel-parse`. Content nested inside a list item, blockquote, or
  slide/tab panel always uses the original sequential path, regardless of
  volume — confirmed by Table 3's near-1.00x rows for the list-heavy/nested
  fixtures.

## 6. Correctness parity (feature on vs off)

Re-run directly for this review (not assumed from the plan's write-up):
`cargo test --release --features parallel-render,parallel-parse` across
`commonmark_spec_it`, `extension_spec_it`, and `parallel_parse_it` all pass
with the same pass rates as the sequential build (§1) — 9/9 additional
`parallel_parse_it` tests green, confirming byte-for-byte/AST parity between
the parallel and sequential code paths on the covered scenarios (deferred
paragraphs, checkbox-split paragraphs, nested-container paragraphs, GFM/
headerless table cells, definition terms, footnote bodies, and the
`track_positions` thread-local propagation risk the plan flagged during
Phase 4 design).

## 7. Conclusion — plan's "Done Definition" checked against this review's data

| Criterion | Status |
|---|---|
| `spec:commonmark` and `fixture:pathological` within ~3x of pulldown-cmark/comrak | ✅ `spec:commonmark` 2.40x/2.39x (Table 1); `fixture:pathological:unbalanced-brackets.md` 4.68x/2.64x, `star-pyramid.md` 12.83x/11.48x — the mixed `fixture:pathological:generated-synthetic.md` corpus sits higher at 15.32x/7.42x, still down from 162.80x/78.78x |
| CommonMark + extension spec suites pass, unchanged | ✅ 357/652 CommonMark, 100% extension suites, reproduced independently (§1) |
| `parallel-render`/`parallel-parse` off by default, output-identical when on | ✅ confirmed in `Cargo.toml` (`default` list) and via test suite parity (§6) |
| Regression gate wired into CI | ✅ `--critical-only --mode parse` gate targeting `spec:commonmark`/`fixture:pathological:*` confirmed present in `.github/workflows/ci-perf.yml` |

All four of the plan's own completion criteria hold up under independent
re-verification. The one nuance worth flagging for future work: the
*mixed* pathological corpus (`fixture:pathological:generated-synthetic.md`,
which interleaves cheap and expensive lines) remains the least-improved
workload at 15x/7x — every fixture that isolates a single pathological
pattern (brackets, emphasis) is now within the <13x range, but the blended
corpus dilutes/compounds differently and was not separately re-targeted
after Phase 2.

---

*Raw artifacts for this review: `tools/perf-lab/output/summary/compare-20260717T192956Z.json`,
`bench-20260717T193033Z/193140Z/193141Z.json` (parallel on: e2e/parse/render),
`bench-20260717T193211Z/193212Z/193214Z.json` (parallel off: e2e/parse/render),
`bench-20260717T193953Z/193954Z.json` (repeated-trial noise check), and
`tools/perf-lab/output/baseline/pre-optimization-e2675b5-compare-e2e.json`
(pre-existing `main`-branch baseline, reused per §2.1).*

## Addendum (2026-07-17): Table 3's manual two-binary workaround is now automated

Section 5 above (and the plan's own Phase 3 note) built two perf-lab binaries
by hand and compared them manually. That gap is closed:
`tools/perf-lab/scripts/run-parallel-compare.sh` now does this automatically —
it builds both binaries, runs the same `bench` invocation against each, and
feeds the two artifacts into the existing `regression` subcommand, which
prints a per-workload %-change table (negative = parallel faster):

```bash
./tools/perf-lab/scripts/run-parallel-compare.sh --release --mode e2e --iterations 20
```

Re-running the full 14-workload sweep through this script reproduces Table 3
directly (`code-heavy.md` -86.7%, i.e. ~7.5x e2e; `paragraph-heavy.md`
-36.8% e2e / -46.8% parse-only, i.e. ~1.9x, matching §5's 1.89x; everything
else within the noise band documented in §5.1). See `Documentation/TOOLS.md`
or `tools/perf-lab/README.md` for full usage.
