//! Precomputed `[`/`]` bracket matching, shared across the link, image, and
//! reference-link grammar.
//!
//! Each of `cm_link::link`, `cm_image::image`, and the reference-link
//! parser's `find_matching_closing_bracket` used to independently scan
//! forward from whatever `[`/`![` position they were called at to find the
//! matching close bracket. Because the inline dispatch loop retries these
//! parsers at every subsequent `[` position when a match attempt fails
//! (e.g. nested/unbalanced brackets with no following `(`), that scan was
//! redone — largely over the same overlapping text — once per opener,
//! giving O(n^2) behavior on adversarial input (see
//! `tools/perf-lab/fixtures/pathological/unbalanced-brackets.md`, ~1000x
//! slower than comparable engines before this fix).
//!
//! [`BracketCacheGuard`] computes all matches for a span exactly once, via
//! one left-to-right (or right-to-left, for the image case) pass, and the
//! three `cached_*` lookups below turn each retry into a lookup over that
//! precomputed list instead of a fresh text scan. Every lookup mirrors its
//! caller's existing per-attempt algorithm exactly (verified: a global
//! stack-based scan produces, for every individual opener, the same match a
//! from-that-opener scan would — nesting/escaping are both
//! position-independent properties of the characters in between), so this
//! is a pure accelerant with no behavior change. Callers still fall back to
//! their original live-scan code when no cache is installed (e.g. calling
//! the grammar function directly, as the unit tests do).
//!
//! Matches are stored as a plain `Vec<(usize, usize)>` with linear lookup
//! rather than a `HashMap`: real documents have at most a handful of
//! bracket pairs per span, where `HashMap`'s allocation and hashing
//! overhead measurably outweighs its O(1) lookup — confirmed by an earlier
//! version of this cache regressing bracket-light workloads like
//! `spec:commonmark` by ~20% despite fixing the pathological case. A linear
//! scan even over the *pathological* fixture's few hundred entries is still
//! orders of magnitude cheaper than the O(remaining-text-length) scan it
//! replaces.

use std::cell::RefCell;
use std::rc::Rc;

struct BracketCaches {
    /// `[` position -> matching `]` position. No escape handling, nested
    /// brackets counted. Mirrors `cm_link::link`'s scan.
    naive_nested: Vec<(usize, usize)>,
    /// `[` position -> matching `]` position. `\[`/`\]` treated as literal,
    /// nested brackets counted. Mirrors the reference-link parser's
    /// `find_matching_closing_bracket(.., allow_nested_brackets = true)`.
    escape_aware_nested: Vec<(usize, usize)>,
    /// `!` position (of `![`) -> position of the next `]` anywhere after
    /// it, ignoring nesting/escapes entirely. Mirrors `cm_image::image`'s
    /// plain `.find(']')` scan.
    image_flat: Vec<(usize, usize)>,
}

fn lookup(entries: &[(usize, usize)], key: usize) -> Option<usize> {
    entries
        .iter()
        .find(|(open, _)| *open == key)
        .map(|(_, close)| *close)
}

thread_local! {
    static CACHE: RefCell<Option<Rc<BracketCaches>>> = const { RefCell::new(None) };
}

/// Installs a freshly computed bracket-match cache for `text` (whose first
/// byte is at absolute offset `base_offset`) for the lifetime of this guard.
/// Restores whatever was previously installed on drop, so nested/recursive
/// `parse_inlines_from_span` calls (e.g. link text) each get their own
/// correctly-scoped cache.
pub struct BracketCacheGuard {
    prev: Option<Rc<BracketCaches>>,
}

impl BracketCacheGuard {
    pub fn install(base_offset: usize, text: &str) -> Self {
        // Every lookup this cache serves is only ever reached when the
        // caller has already confirmed the current position starts with
        // `[` (or `![`) — so if `text` has no `[` at all, none of the three
        // lists could have any entries, and none of the lookups would ever
        // be attempted regardless. Skip the three O(n) scans entirely in
        // that case (the common case for bracket-free prose) — leaving no
        // cache installed is equivalent to an empty one here, and callers
        // already handle "no cache" via their live-scan fallback, which is
        // unreachable for the same reason.
        let prev = if text.contains('[') {
            let caches = Rc::new(BracketCaches {
                naive_nested: scan_nested(text, base_offset, false),
                escape_aware_nested: scan_nested(text, base_offset, true),
                image_flat: scan_image_flat(text, base_offset),
            });
            CACHE.with(|c| c.borrow_mut().replace(caches))
        } else {
            CACHE.with(|c| c.borrow_mut().take())
        };
        Self { prev }
    }
}

impl Drop for BracketCacheGuard {
    fn drop(&mut self) {
        CACHE.with(|c| *c.borrow_mut() = self.prev.take());
    }
}

/// One left-to-right pass computing, for every unescaped `[` in `text`, the
/// offset of its properly nested matching `]` (or no entry if unmatched).
fn scan_nested(text: &str, base_offset: usize, respect_escapes: bool) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut escaped = false;
    let mut i = 0usize;

    for ch in text.chars() {
        if respect_escapes {
            if escaped {
                escaped = false;
                i += ch.len_utf8();
                continue;
            }
            if ch == '\\' {
                escaped = true;
                i += ch.len_utf8();
                continue;
            }
        }

        match ch {
            '[' => stack.push(base_offset + i),
            ']' => {
                if let Some(open) = stack.pop() {
                    matches.push((open, base_offset + i));
                }
            }
            _ => {}
        }

        i += ch.len_utf8();
    }

    matches
}

/// One right-to-left pass computing, for every `![` in `text`, the offset
/// of the next `]` anywhere after it (no nesting/escape awareness, matching
/// `cm_image::image`'s plain `.find(']')`).
fn scan_image_flat(text: &str, base_offset: usize) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    let bytes = text.as_bytes();
    let mut next_close: Option<usize> = None;
    let mut i = text.len();

    while i > 0 {
        i -= 1;
        if bytes[i] == b']' {
            next_close = Some(base_offset + i);
        }
        if bytes[i] == b'!' && bytes.get(i + 1) == Some(&b'[') {
            if let Some(close) = next_close {
                matches.push((base_offset + i, close));
            }
        }
    }

    matches
}

/// `Some(inner)` when a cache is installed (`inner` is the cached match, if
/// any); `None` when no cache is installed — callers must fall back to a
/// live scan in that case only (a `Some(None)` means the cache confirms
/// there is no match, and callers should trust that directly).
pub fn cached_naive_nested_match(open_abs_offset: usize) -> Option<Option<usize>> {
    CACHE.with(|c| {
        c.borrow()
            .as_ref()
            .map(|m| lookup(&m.naive_nested, open_abs_offset))
    })
}

/// See [`cached_naive_nested_match`] for the `Option<Option<_>>` contract.
pub fn cached_escape_aware_nested_match(open_abs_offset: usize) -> Option<Option<usize>> {
    CACHE.with(|c| {
        c.borrow()
            .as_ref()
            .map(|m| lookup(&m.escape_aware_nested, open_abs_offset))
    })
}

/// See [`cached_naive_nested_match`] for the `Option<Option<_>>` contract.
pub fn cached_image_flat_match(bang_abs_offset: usize) -> Option<Option<usize>> {
    CACHE.with(|c| {
        c.borrow()
            .as_ref()
            .map(|m| lookup(&m.image_flat, bang_abs_offset))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_nested_matches_independent_per_opener_scans() {
        // "[[[unbalanced]]" — 3 opens, 2 closes: opener 0 has no match,
        // opener 1 matches the last `]`, opener 2 matches the first `]`.
        let text = "[[[unbalanced]]";
        let matches = scan_nested(text, 0, false);
        assert_eq!(lookup(&matches, 0), None);
        assert_eq!(lookup(&matches, 1), Some(14));
        assert_eq!(lookup(&matches, 2), Some(13));
    }

    #[test]
    fn scan_nested_respects_escapes() {
        let text = r"[a\]b]";
        let escaped = scan_nested(text, 0, true);
        assert_eq!(lookup(&escaped, 0), Some(5));

        let naive = scan_nested(text, 0, false);
        assert_eq!(lookup(&naive, 0), Some(3));
    }

    #[test]
    fn scan_image_flat_ignores_nesting() {
        let text = "![a[b]c]";
        let matches = scan_image_flat(text, 0);
        // First `]` after `![` is at index 5, regardless of the `[` at 3.
        assert_eq!(lookup(&matches, 0), Some(5));
    }

    #[test]
    fn guard_restores_previous_on_drop() {
        assert_eq!(cached_naive_nested_match(0), None);
        {
            let _outer = BracketCacheGuard::install(0, "[foo]");
            assert_eq!(cached_naive_nested_match(0), Some(Some(4)));
            {
                let _inner = BracketCacheGuard::install(100, "[bar]");
                assert_eq!(cached_naive_nested_match(100), Some(Some(104)));
                // A cache is installed (the inner one), so offset 0 — which
                // belongs to the outer span, not this one — correctly comes
                // back as "installed, no match" rather than "no cache".
                assert_eq!(cached_naive_nested_match(0), Some(None));
            }
            assert_eq!(cached_naive_nested_match(0), Some(Some(4)));
        }
        assert_eq!(cached_naive_nested_match(0), None);
    }

    #[test]
    fn guard_skips_installation_when_no_bracket_present() {
        assert_eq!(cached_naive_nested_match(0), None);
        let _guard = BracketCacheGuard::install(0, "plain prose, no brackets here");
        // No `[` anywhere: nothing installed, still reads as "no cache".
        assert_eq!(cached_naive_nested_match(0), None);
    }
}
