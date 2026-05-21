#![cfg(feature = "intelligence-highlights")]
use marco_core::intelligence::{compute_highlights, compute_highlights_with_source, HighlightTag};
use marco_core::parser::{Document, Node, NodeKind, Position, Span};

#[test]
fn test_compute_highlights_multi_byte() {
    // Build a document with several nodes including multi-byte content
    let doc = Document {
        children: vec![
            Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "Title Café".to_string(),
                    id: None,
                },
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 1,
                        column: 12,
                        offset: 11,
                    },
                }),
                children: vec![],
            },
            Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Emphasis,
                        span: Some(Span {
                            start: Position {
                                line: 2,
                                column: 1,
                                offset: 12,
                            },
                            end: Position {
                                line: 2,
                                column: 7,
                                offset: 18,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Strong,
                        span: Some(Span {
                            start: Position {
                                line: 2,
                                column: 9,
                                offset: 20,
                            },
                            end: Position {
                                line: 2,
                                column: 15,
                                offset: 26,
                            },
                        }),
                        children: vec![],
                    },
                ],
            },
        ],
        ..Default::default()
    };

    let highlights = compute_highlights(&doc);

    // Expect at least heading, emphasis and strong highlights
    assert!(highlights.iter().any(|h| h.tag == HighlightTag::Heading1));
    assert!(highlights.iter().any(|h| h.tag == HighlightTag::Emphasis));
    assert!(highlights.iter().any(|h| h.tag == HighlightTag::Strong));

    // Ensure heading highlight was expanded to line start (column == 1)
    if let Some(h) = highlights.iter().find(|h| h.tag == HighlightTag::Heading1) {
        assert_eq!(h.span.start.column, 1);
        assert_eq!(h.span.start.line, 1);
    } else {
        panic!("Missing Heading1 highlight");
    }
}

#[test]
fn test_compute_highlights_with_source_adds_tab_block_markers() {
    use marco_core::parse;

    let source = ":::tab\n@tab First\nContent A\n@tab Second\nContent B\n:::\n";
    let doc = parse(source).expect("parse failed");

    // compute_highlights only produces AST-derived highlights
    let ast_highlights = compute_highlights(&doc);
    // compute_highlights_with_source additionally scans source for tab block markers
    let source_highlights = compute_highlights_with_source(&doc, source);

    // The with_source variant must produce at least as many highlights
    assert!(
        source_highlights.len() >= ast_highlights.len(),
        "with_source should never produce fewer highlights than the AST-only variant"
    );

    // Should contain at least one TabBlockContainer or TabBlockHeader tag
    let has_tab_marker = source_highlights
        .iter()
        .any(|h| h.tag == HighlightTag::TabBlockContainer || h.tag == HighlightTag::TabBlockHeader);
    assert!(
        has_tab_marker,
        "expected tab-block marker highlights from source scan; got: {source_highlights:?}"
    );
}

#[test]
fn test_compute_highlights_with_source_adds_slider_markers() {
    use marco_core::parse;

    let source = "@slidestart:t5\n# Slide One\n---\n# Slide Two\n@slideend\n";
    let doc = parse(source).expect("parse failed");

    let source_highlights = compute_highlights_with_source(&doc, source);

    let has_slider = source_highlights.iter().any(|h| {
        matches!(
            h.tag,
            HighlightTag::SliderDeckMarker | HighlightTag::SliderSeparatorHorizontal
        )
    });
    assert!(
        has_slider,
        "expected slider marker highlights from source scan; got: {source_highlights:?}"
    );
}

#[test]
fn test_compute_highlights_with_source_matches_ast_only_for_plain_markdown() {
    use marco_core::parse;

    // Plain markdown has no tab/slider markers → both variants must produce identical results
    let source = "# Heading\n\nA paragraph with **bold** and *italic*.\n";
    let doc = parse(source).expect("parse failed");

    let ast_only = compute_highlights(&doc);
    let with_source = compute_highlights_with_source(&doc, source);

    assert_eq!(
        ast_only, with_source,
        "for plain markdown without marcos, both highlight variants should be identical"
    );
}
