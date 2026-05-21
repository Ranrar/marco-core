//! Integration tests for the `MarkdownIntelligenceProvider` facade
//! re-exported from `lib.rs`.

#![cfg(all(
    feature = "intelligence-highlights",
    feature = "intelligence-diagnostics",
    feature = "intelligence-completions",
    feature = "intelligence-hover"
))]

use marco_core::{parse, MarkdownIntelligenceProvider};

#[test]
fn test_intelligence_provider_yields_highlights_for_parsed_document() {
    let md = "# Heading\n\nA paragraph with **bold** text.\n";
    let doc = parse(md).expect("parse failed");

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc);

    let highlights = provider.highlights(md);
    assert!(
        !highlights.is_empty(),
        "expected at least one highlight tag for a heading + bold paragraph"
    );
}

#[test]
fn test_intelligence_provider_returns_diagnostics_vec() {
    let md = "# OK\n";
    let doc = parse(md).expect("parse failed");

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc);

    // We don't assert on specific diagnostics — only that the public
    // API call succeeds and returns a Vec we can iterate.
    let diags = provider.diagnostics();
    let _len = diags.len();
}

#[test]
fn test_intelligence_provider_completions_callable() {
    let md = "# OK\n";
    let doc = parse(md).expect("parse failed");

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc);

    let _ = provider.completions("");
}

#[test]
fn test_intelligence_provider_diagnostics_with_options_filters_by_severity() {
    use marco_core::intelligence::DiagnosticsOptions;

    // Produce a document that triggers at least a warning-level diagnostic
    // (bare URL without autolink formatting is a common warning).
    let md = "Visit https://example.com for more.\n";
    let doc = parse(md).expect("parse failed");

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc);

    let all_diags = provider.diagnostics();
    let critical_only = provider.diagnostics_with_options(DiagnosticsOptions::critical_only());

    // critical_only must never return more diagnostics than the full set
    assert!(
        critical_only.len() <= all_diags.len(),
        "critical_only filter must not add diagnostics; all={} critical={}",
        all_diags.len(),
        critical_only.len()
    );
}

#[test]
fn test_intelligence_provider_hover_returns_info_for_link() {
    use marco_core::parser::Position;

    // Parse a document with an inline link on line 1
    let md = "[label](https://example.com)\n";
    let doc = parse(md).expect("parse failed");

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc);

    // Position inside the link text "[label]" — column 2, line 1
    let pos = Position {
        line: 1,
        column: 2,
        offset: 1,
    };

    // hover() must not panic and returns Some for a position inside a link
    let info = provider.hover(pos);
    assert!(
        info.is_some(),
        "expected hover info for a position inside a link label"
    );
}

#[test]
fn test_intelligence_provider_hover_returns_none_outside_document() {
    use marco_core::parser::Position;

    let md = "# Heading\n";
    let doc = parse(md).expect("parse failed");

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc);

    // Position well beyond the document — should return None without panicking
    let pos = Position {
        line: 999,
        column: 1,
        offset: 9999,
    };
    let info = provider.hover(pos);
    assert!(
        info.is_none(),
        "hover outside the document bounds should return None"
    );
}
