//! Integration tests for the `MarkdownIntelligenceProvider` facade
//! re-exported from `lib.rs`.

use marco_core::{parse, MarkdownIntelligenceProvider};

#[test]
fn intelligence_provider_yields_highlights_for_parsed_document() {
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
fn intelligence_provider_returns_diagnostics_vec() {
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
fn intelligence_provider_completions_callable() {
    let md = "# OK\n";
    let doc = parse(md).expect("parse failed");

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc);

    let _ = provider.completions("");
}
