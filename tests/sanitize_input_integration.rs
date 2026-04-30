//! Integration tests for the UTF-8 sanitization helpers re-exported from
//! `lib.rs`: `sanitize_input`, `sanitize_input_with_stats`, `InputSource`,
//! `SanitizeStats`.

use marco_core::{sanitize_input, sanitize_input_with_stats, InputSource};

#[test]
fn sanitize_input_passes_clean_utf8_through() {
    let input = "Hello, world!\n";
    let out = sanitize_input(input.as_bytes(), InputSource::Keyboard);
    assert_eq!(out, input);
}

#[test]
fn sanitize_input_strips_null_bytes() {
    let input = b"abc\0def";
    let out = sanitize_input(input, InputSource::File);
    assert!(!out.contains('\0'), "null bytes must be removed: {out:?}");
    assert!(out.contains("abc"));
    assert!(out.contains("def"));
}

#[test]
fn sanitize_input_replaces_invalid_utf8() {
    // 0xFF on its own is not valid UTF-8.
    let input = b"good\xFFbad";
    let out = sanitize_input(input, InputSource::Network);
    // Sanitizer must produce valid UTF-8 either by replacing or dropping
    // the invalid byte; the surrounding ASCII must survive.
    assert!(out.is_char_boundary(out.len()));
    assert!(out.contains("good"));
    assert!(out.contains("bad"));
}

#[test]
fn sanitize_input_with_stats_reports_byte_counts() {
    let input = b"hello\0world";
    let (out, stats) = sanitize_input_with_stats(input, InputSource::Clipboard);

    assert!(!out.contains('\0'));
    assert_eq!(stats.original_bytes, input.len());
    assert_eq!(stats.sanitized_bytes, out.len());
    assert!(stats.had_issues(), "null byte should register as an issue");
}
