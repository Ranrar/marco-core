use assert_cmd::Command;

#[test]
fn emits_valid_json_report() {
    let output = Command::cargo_bin("marco-ast")
        .expect("binary")
        .args([
            "--text",
            "# Tëst\\n\\nHello  \\nworld",
            "--mode",
            "both",
            "--json",
            "--spans",
            "--utf8",
            "--time",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: serde_json::Value = serde_json::from_slice(&output).expect("valid json");
    assert_eq!(value["mode"], "both");
    assert!(value["timing"]["parse_us"].is_number());
    assert!(value["sanitize_stats"]["sanitized_bytes"].is_number());
    assert!(value["span_samples"].is_array());
}

#[test]
fn shows_span_and_excerpt_in_text_mode() {
    let output = Command::cargo_bin("marco-ast")
        .expect("binary")
        .args([
            "--text",
            "Hello  \nworld",
            "--mode",
            "ast",
            "--spans",
            "--excerpts",
            "--no-color",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).expect("utf8");
    assert!(text.contains("bytes "));
    assert!(text.contains("«Hello  ↵world»"));
}
