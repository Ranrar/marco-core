# API Naming Standard

These rules govern all runtime-visible identifiers produced by `marco-core`.
Every new feature and every rename must comply with these rules before merging.

---

## Rules

### Rule 1 — Rust symbols

- Marco extension modules and types **must** carry the `marco_` / `Marco` prefix matching their source file name.
  - `marco_sliders.rs` → `MarcoSlideDeck`, `MarcoSlide`, `marco_slide_deck()`
  - `marco_tab_blocks.rs` → `MarcoTabBlock`, `MarcoTabItem`, `marco_tab_block()`
  - `marco_headerless_table.rs` → `MarcoHeaderlessTableBlock`, `marco_headerless_table()`
- The file-naming prefix taxonomy **must** stay consistent: `cm_*` (CommonMark), `gfm_*` (GFM), `marco_*` (Marco extensions).
- Top-level public API re-exported from `src/lib.rs` **must** use neutral, concise names (`parse`, `render`, `RenderOptions`, …).

### Rule 2 — JavaScript bridge

- The single global bridge **must** be `window.MarcoCorePreview`.
- No generic globals (`PreviewBridge`, `Preview`, etc.) are permitted.
- Methods on the bridge follow camelCase: `MarcoCorePreview.updateContent()`, `MarcoCorePreview.cleanup()`.

### Rule 3 — HTML element IDs

All IDs emitted by the renderer **must** use the pattern `mc-<feature>-<part>`.

| ID | Element |
|---|---|
| `mc-content-container` | main rendered-content `<div>` |
| `mc-preview-style` | external preview stylesheet `<style>` |
| `mc-preview-internal-style` | internal preview stylesheet `<style>` |
| `mc-paged-page-css` | paged-view stylesheet `<style>` |
| `mc-print-export-css` | print-export stylesheet `<style>` |

### Rule 4 — CSS classes

All CSS classes emitted by the renderer **must** follow `marco-<feature>` BEM blocks.
The `marco-` prefix is required on the block name; BEM elements (`__`) and modifiers (`--`) follow standard BEM convention.

| Feature | Block | Example elements / modifiers |
|---|---|---|
| Sliders | `marco-sliders` | `__viewport`, `__slide`, `__controls`, `__btn`, `__btn--prev`, `__btn--next`, `__btn--toggle`, `__icon`, `__icon--play`, `__icon--pause`, `__dots`, `__dot`, `__dot-icon`, `__dot-icon--active`, `__dot-icon--inactive` |
| Tab blocks | `marco-tabs` | `__radio`, `__tablist`, `__tab`, `__panels`, `__panel` |
| Task list | `marco-task-list-item`, `marco-task-checkbox`, `marco-task-icon`, `marco-task-check`, `marco-task-box` | `marco-task-list-item--checked` |
| Code blocks | `marco-code-block`, `marco-copy-btn` | — |
| Headings | `marco-heading-anchor` | — |
| Platform mentions | `marco-mention`, `marco-mention-<platform>` | — |
| Autolinks | `marco-autolink` | — |
| Inline footnotes | `marco-inline-footnote` | — |
| Emoji | `marco-emoji` | — |
| Diagrams | `marco-diagram` | — |
| Tables | `marco-table-auto-align`, `marco-table-resizing`, `marco-resize-active` | — |

**Exception:** `nested-code-block` and its sub-classes (`nested-code-block.level-N`, `.code-header`, `.code-content`) are structural helpers and are intentionally unprefixed.

### Rule 5 — CSS custom properties

All custom properties **must** use the pattern `--mc-<feature>-<token>`.

| Property | Purpose |
|---|---|
| `--mc-task-primary` | Task checkbox primary colour |
| `--mc-task-accent` | Task checkbox accent / check colour |
| `--mc-sliders-border` | Slider deck border |
| `--mc-sliders-bg` | Slider deck background |

### Rule 6 — DOM / data signal values

String values used as signals (e.g. `document.title` handshakes) **must** use `mc_*` snake_case.

| Signal | Use |
|---|---|
| `mc_paged_ready` | Set as `document.title` to signal paged-view ready |

---

## Checklist for New Features

When adding a new Marco extension block (e.g. `marco_foo.rs`):

1. Grammar struct/fn: `MarcoFoo` / `marco_foo()`
2. CSS block class: `marco-foo`
3. CSS elements/modifiers: `marco-foo__part`, `marco-foo__part--state`
4. CSS vars (if any): `--mc-foo-token`
5. HTML ID (if any): `mc-foo-part`
6. Add selectors to `src/render/base_css.rs`
7. Emit classes in `src/render/markdown.rs`
8. Add/update integration test asserting the canonical class names
9. Document in `CHANGELOG.md` under `[Unreleased] → Added`

---

## SemVer Implications

- Adding new `marco-*` classes or `--mc-*` vars → **minor** bump.
- Renaming or removing an existing public class/var/ID/JS method → **major** bump.
- Internal Rust symbol renames that are not re-exported from `src/lib.rs` → **patch** bump.

Use `#[deprecated(since = "X.Y.Z", note = "use <new_name>")]` for any Rust symbol that was previously public and is being renamed.
