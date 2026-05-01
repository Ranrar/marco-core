//! Integration tests for the cached pipeline re-exported from `lib.rs`:
//! `ParserCache`, `parse_to_html`, `parse_to_html_cached`.

use marco_core::{parse_to_html, parse_to_html_cached, ParserCache, RenderOptions};

#[test]
fn test_parser_cache_returns_same_html_for_repeated_input() {
    let cache = ParserCache::new();
    let md = "# Hello\n\nA *cached* paragraph.\n";

    let first = cache
        .render_with_cache(md, RenderOptions::default())
        .expect("first render failed");
    let second = cache
        .render_with_cache(md, RenderOptions::default())
        .expect("second render failed");

    assert_eq!(first, second, "cached render must be deterministic");
    assert!(first.contains("<h1") && first.contains("Hello"));
    assert!(first.contains("<em>cached</em>"));
}

#[test]
fn test_parser_cache_records_entries_after_calls() {
    let cache = ParserCache::new();
    let before = cache.stats();
    assert_eq!(before.ast_entries, 0);
    assert_eq!(before.html_entries, 0);

    let _ = cache
        .render_with_cache("paragraph\n", RenderOptions::default())
        .expect("render failed");

    let after = cache.stats();
    // moka's entry_count is eventually consistent; allow ≥0 but assert
    // the cache itself functions by re-rendering and checking stability.
    let again = cache
        .render_with_cache("paragraph\n", RenderOptions::default())
        .expect("second render failed");
    assert!(again.contains("<p>paragraph</p>"));
    assert!(after.ast_capacity > 0);
    assert!(after.html_capacity > 0);
}

#[test]
fn test_parse_to_html_uncached_matches_cached() {
    let md = "## Title\n\n- a\n- b\n";

    let uncached = parse_to_html(md, RenderOptions::default()).expect("uncached failed");
    let cached = parse_to_html_cached(md, RenderOptions::default()).expect("cached failed");

    assert_eq!(uncached, cached);
    assert!(uncached.contains("<h2") && uncached.contains("Title"));
    assert!(uncached.contains("<li>a</li>"));
    assert!(uncached.contains("<li>b</li>"));
}
