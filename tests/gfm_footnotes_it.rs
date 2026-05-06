use marco_core::parser::parse;
use marco_core::render::RenderOptions;

#[test]
fn test_gfm_footnotes_basic_rendering() {
    let input = "Here is a footnote reference[^1].\n\n[^1]: Footnote definition.\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    // Reference becomes a numbered superscript link.
    assert!(html.contains("<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup>"));

    // Definitions should not render in place.
    assert!(!html.contains("[^1]:"));

    // Footnotes section is appended.
    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("<li id=\"fn1\">"));
    assert!(html.contains("Footnote definition."));
}

#[test]
fn test_gfm_footnotes_missing_definition_falls_back_to_literal() {
    let input = "Missing def[^missing].\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    // No footnotes section, and the original marker remains.
    assert!(!html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("[^missing]"));
}

#[test]
fn test_gfm_footnotes_multiline_definition() {
    let input =
        "A multi-line footnote[^multi].\n\n[^multi]: First line\n    second line\n    third line\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<section class=\"footnotes\">"));
    assert!(html.contains("First line"));
    assert!(html.contains("second line"));
    assert!(html.contains("third line"));
}

#[test]
fn test_gfm_footnotes_multiparagraph_definition() {
    // A blank line followed by a 4-space-indented continuation must be consumed
    // as part of the footnote definition, not parsed as an indented code block.
    let input = "See[^1]\n\n[^1]: First para.\n\n    Second para.\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("<section class=\"footnotes\">"));
    // Both paragraphs must appear inside the footnote section.
    assert!(html.contains("First para."), "first para lost: {html}");
    assert!(html.contains("Second para."), "second para lost: {html}");
    // The continuation must NOT become a standalone indented code block.
    assert!(
        !html.contains("marco-code-block"),
        "continuation became code block: {html}"
    );
}

#[test]
fn test_gfm_footnotes_multiparagraph_three_paras() {
    let input = "Ref[^x]\n\n[^x]: Para one.\n\n    Para two.\n\n    Para three.\n\nNormal text.\n";
    let doc = parse(input).expect("parse failed");
    let options = RenderOptions::default();
    let html = marco_core::render::render(&doc, &options).expect("render failed");

    assert!(html.contains("Para one."), "para one lost");
    assert!(html.contains("Para two."), "para two lost");
    assert!(html.contains("Para three."), "para three lost");
    assert!(!html.contains("marco-code-block"), "continuation became code block");
    // Normal text after the footnote block must still render.
    assert!(html.contains("Normal text."), "normal text lost");
}
