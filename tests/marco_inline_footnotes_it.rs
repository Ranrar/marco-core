use marco_core::parser::parse;
use marco_core::render::RenderOptions;

#[test]
fn test_inline_footnote_basic_rendering() {
    let input = "Inline note.^[This is it]\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    // Reference becomes a numbered superscript link.
    assert!(html.contains("<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup>"));

    // Marker is not preserved.
    assert!(!html.contains("^[This is it]"));

    // Footnotes section includes the content.
    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("This is it"));
}

#[test]
fn test_inline_footnote_supports_inline_markup() {
    let input = "Complex^[*italic* and **bold** and `code`]\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<code>code</code>"));
}

#[test]
fn test_inline_footnote_not_parsed_inside_code_span() {
    let input = "`^[not a footnote]`\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    assert!(!html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("<code>^[not a footnote]</code>"));
}

#[test]
fn test_inline_footnote_does_not_break_superscript() {
    let input = "^hi^ and a note^[x]\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    // Superscript is rendered normally.
    assert!(html.contains("<sup>hi</sup>"));

    // Footnote still works.
    assert!(html.contains("<section class=\"footnotes\">"));
}

#[test]
fn test_inline_footnote_spans_multiple_lines() {
    // An inline footnote whose content spans multiple lines of the source
    // paragraph must be parsed correctly — not rendered as literal `^[...]`.
    let input = "A fact^[This spans\ntwo lines].\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    assert!(
        html.contains("<sup class=\"footnote-ref\">"),
        "inline fn not parsed (still literal): {html}"
    );
    assert!(!html.contains("^["), "raw ^[ marker in output: {html}");
    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("This spans"));
}

#[test]
fn test_inline_footnote_multiline_with_markup() {
    // Multi-line inline footnote content with inline markup inside.
    let input =
        "Note^[This is defined\ninline. It contains *italic*, **bold**, and `code`.].\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<sup class=\"footnote-ref\">"));
    assert!(html.contains("<em>italic</em>"), "italic lost in multi-line fn");
    assert!(html.contains("<strong>bold</strong>"), "bold lost in multi-line fn");
    assert!(html.contains("<code>code</code>"), "code lost in multi-line fn");
}
