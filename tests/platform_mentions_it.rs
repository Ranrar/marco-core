use marco_core::render::RenderOptions;

#[test]
fn test_platform_mention_github_default_label() {
    let md = "Hello @ranrar[github]!";
    let doc = marco_core::parser::parse(md).expect("parse failed");
    let html = marco_core::render::render(&doc, &RenderOptions::default()).expect("render failed");

    assert!(
        html.contains("<a class=\"marco-mention marco-mention-github\" href=\"https://github.com/ranrar\">ranrar</a>"),
        "unexpected html: {html}"
    );
}

#[test]
fn test_platform_mention_github_display_override() {
    let md = "Hello @ranrar[github](Kim)!";
    let doc = marco_core::parser::parse(md).expect("parse failed");
    let html = marco_core::render::render(&doc, &RenderOptions::default()).expect("render failed");

    assert!(
        html.contains(
            "<a class=\"marco-mention marco-mention-github\" href=\"https://github.com/ranrar\">Kim</a>"
        ),
        "unexpected html: {html}"
    );
}

#[test]
fn test_platform_mention_unknown_platform_renders_span() {
    let md = "Hello @ranrar[unknown](Kim)!";
    let doc = marco_core::parser::parse(md).expect("parse failed");
    let html = marco_core::render::render(&doc, &RenderOptions::default()).expect("render failed");

    assert!(
        html.contains("<span class=\"marco-mention marco-mention-unknown\">Kim</span>"),
        "unexpected html: {html}"
    );
}
