use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
use marco_core::{Document, MarkdownIntelligenceProvider, Node, NodeKind, RenderOptions};

use crate::ast_print::print_ast;
use crate::cli::Args;

/// Result payload for logging.
pub struct RunPayload {
    pub ast: Option<String>,
    pub html: Option<String>,
    pub diagnostics_summary: Option<String>,
}

pub fn print_rule(use_color: bool) {
    if use_color {
        println!(
            "{}{}{}",
            SetForegroundColor(Color::DarkGrey),
            "═".repeat(50),
            ResetColor
        );
    } else {
        println!("{}", "═".repeat(50));
    }
}

fn print_section_header(title: &str, use_color: bool) {
    if use_color {
        println!(
            "{}{}═══ {title} {}{}",
            SetForegroundColor(Color::DarkGrey),
            SetAttribute(Attribute::Dim),
            "═".repeat(46usize.saturating_sub(title.len())),
            ResetColor,
        );
    } else {
        println!("═══ {title} {}", "═".repeat(46usize.saturating_sub(title.len())));
    }
}

// ── Stray delimiter collection ────────────────────────────────────────────────

struct StrayDelimiter {
    text: String,
    /// 1-based line number from the span, or 0 if unknown.
    line: usize,
    col: usize,
}

fn collect_stray_delimiters(nodes: &[Node], out: &mut Vec<StrayDelimiter>) {
    for node in nodes {
        if let NodeKind::Text(s) = &node.kind {
            let trimmed = s.trim();
            if !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| matches!(c, '*' | '_' | '~' | '=' | '^' | '`'))
            {
                let (line, col) = node
                    .span
                    .map(|sp| (sp.start.line, sp.start.column))
                    .unwrap_or((0, 0));
                out.push(StrayDelimiter {
                    text: trimmed.to_string(),
                    line,
                    col,
                });
            }
        }
        collect_stray_delimiters(&node.children, out);
    }
}

// ── Source context helpers ────────────────────────────────────────────────────

/// Return the source line at `line` (1-based), capped to `max` chars.
fn source_line(source: &str, line: usize) -> Option<&str> {
    if line == 0 {
        return None;
    }
    source.lines().nth(line.saturating_sub(1))
}

fn caret_line(col: usize, len: usize) -> String {
    let indent = col.saturating_sub(1);
    let width = len.max(1);
    format!("{}{}", " ".repeat(indent), "^".repeat(width))
}

// ── Severity helpers ─────────────────────────────────────────────────────────

fn sev_color(severity: &str) -> Color {
    match severity {
        "Error" => Color::Red,
        "Warning" => Color::Yellow,
        "Information" => Color::Cyan,
        "Hint" => Color::DarkGrey,
        _ => Color::White,
    }
}

// ── Main diagnostics printer ─────────────────────────────────────────────────

/// Runs intelligence diagnostics + stray-delimiter scan and prints a
/// compiler-style DIAGNOSTICS section.  Always called from ast/html/both modes.
/// Returns a summary string for the log payload.
fn print_diagnostics_section(doc: &Document, source: &str, use_color: bool) -> String {
    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc.clone());
    let diagnostics = provider.diagnostics();

    // Stray delimiter warnings from the AST
    let mut strays: Vec<StrayDelimiter> = Vec::new();
    collect_stray_delimiters(&doc.children, &mut strays);

    let total = diagnostics.len() + strays.len();

    if total == 0 {
        // Clean — show a brief green footer
        if use_color {
            println!(
                "{}{}✓ no issues{}",
                SetForegroundColor(Color::Green),
                SetAttribute(Attribute::Dim),
                ResetColor,
            );
        } else {
            println!("✓ no issues");
        }
        return "0 issue(s)".to_string();
    }

    println!();
    if use_color {
        println!(
            "{}{}─── DIAGNOSTICS ({total}) {}{}",
            SetForegroundColor(Color::DarkGrey),
            SetAttribute(Attribute::Dim),
            "─".repeat(36usize.saturating_sub(total.to_string().len())),
            ResetColor,
        );
    } else {
        println!(
            "─── DIAGNOSTICS ({total}) {}",
            "─".repeat(36usize.saturating_sub(total.to_string().len()))
        );
    }

    // Intelligence diagnostics
    for d in &diagnostics {
        let severity = format!("{:?}", d.severity);
        let color = sev_color(&severity);
        let line = d.span.start.line;
        let col = d.span.start.column;

        if use_color {
            println!(
                "  {}{}[{severity}]{} {}  (line {line}, col {col})",
                SetAttribute(Attribute::Bold),
                SetForegroundColor(color),
                ResetColor,
                d.message,
            );
        } else {
            println!("  [{severity}] {}  (line {line}, col {col})", d.message);
        }

        // Source line + caret
        if let Some(src_line) = source_line(source, line) {
            let span_len = d.span.end.offset.saturating_sub(d.span.start.offset);
            let display_len = span_len.min(src_line.len().saturating_sub(col.saturating_sub(1)));
            if use_color {
                println!("    {} │ {src_line}", line);
                println!(
                    "    {} │ {}{}{}",
                    " ".repeat(line.to_string().len()),
                    SetForegroundColor(color),
                    caret_line(col, display_len.max(1)),
                    ResetColor,
                );
            } else {
                println!("    {line} │ {src_line}");
                println!("    {} │ {}", " ".repeat(line.to_string().len()), caret_line(col, display_len.max(1)));
            }
        }
    }

    // Stray delimiter warnings
    for s in &strays {
        let color = Color::Yellow;
        if use_color {
            println!(
                "  {}{}[Warning]{} unmatched delimiter {:?}  (line {}, col {})",
                SetAttribute(Attribute::Bold),
                SetForegroundColor(color),
                ResetColor,
                s.text,
                s.line,
                s.col,
            );
        } else {
            println!(
                "  [Warning] unmatched delimiter {:?}  (line {}, col {})",
                s.text, s.line, s.col,
            );
        }

        if s.line > 0 {
            if let Some(src_line) = source_line(source, s.line) {
                if use_color {
                    println!("    {} │ {src_line}", s.line);
                    println!(
                        "    {} │ {}{}{}",
                        " ".repeat(s.line.to_string().len()),
                        SetForegroundColor(color),
                        caret_line(s.col, s.text.len()),
                        ResetColor,
                    );
                } else {
                    println!("    {} │ {src_line}", s.line);
                    println!(
                        "    {} │ {}",
                        " ".repeat(s.line.to_string().len()),
                        caret_line(s.col, s.text.len()),
                    );
                }
            }
        }
    }

    format!("{total} issue(s)")
}

pub fn run_ast_mode(doc: &Document, source: &str, args: &Args) -> RunPayload {
    print_section_header("AST", !args.no_color);
    let tree = print_ast(doc, !args.no_color, args.compact);
    print!("{tree}");
    let diag_summary = print_diagnostics_section(doc, source, !args.no_color);
    RunPayload {
        ast: Some(tree),
        html: None,
        diagnostics_summary: Some(diag_summary),
    }
}

pub fn run_html_mode(doc: &Document, source: &str, args: &Args) -> RunPayload {
    let options = RenderOptions {
        syntax_highlighting: args.syntax,
        ..RenderOptions::default()
    };
    match marco_core::render(doc, &options) {
        Ok(html) => {
            print_section_header("HTML", !args.no_color);
            println!("{html}");
            let diag_summary = print_diagnostics_section(doc, source, !args.no_color);
            RunPayload {
                ast: None,
                html: Some(html),
                diagnostics_summary: Some(diag_summary),
            }
        }
        Err(e) => {
            eprintln!("render error: {e}");
            RunPayload { ast: None, html: None, diagnostics_summary: None }
        }
    }
}

pub fn run_intel_mode(doc: &Document, source: &str, args: &Args) -> RunPayload {
    print_section_header("INTELLIGENCE", !args.no_color);

    let mut provider = MarkdownIntelligenceProvider::new();
    provider.update_document(doc.clone());

    let diagnostics = provider.diagnostics();
    let highlights = provider.highlights(source);

    let use_color = !args.no_color;

    if diagnostics.is_empty() {
        if use_color {
            println!(
                "{}{}(no diagnostics){}",
                SetForegroundColor(Color::DarkGrey),
                SetAttribute(Attribute::Dim),
                ResetColor,
            );
        } else {
            println!("(no diagnostics)");
        }
    } else {
        for d in &diagnostics {
            let severity = format!("{:?}", d.severity);
            let color = sev_color(&severity);
            if use_color {
                println!(
                    "[{}{}{}] {}  ({}:{})",
                    SetForegroundColor(color),
                    severity,
                    ResetColor,
                    d.message,
                    d.span.start.line,
                    d.span.start.column,
                );
            } else {
                println!(
                    "[{severity}] {}  ({}:{})",
                    d.message,
                    d.span.start.line,
                    d.span.start.column,
                );
            }
        }
    }

    println!();

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for h in &highlights {
        *counts.entry(format!("{:?}", h.tag)).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    println!("Highlight spans ({} total):", highlights.len());
    for (tag, count) in sorted {
        println!("  {tag}: {count}");
    }

    let summary = format!(
        "{} diagnostic(s), {} highlight span(s)",
        diagnostics.len(),
        highlights.len()
    );

    RunPayload {
        ast: None,
        html: None,
        diagnostics_summary: Some(summary),
    }
}

pub fn run_both_mode(doc: &Document, source: &str, args: &Args) -> RunPayload {
    let ast_payload = run_ast_mode(doc, source, args);
    print_rule(!args.no_color);
    let html_payload = run_html_mode(doc, source, args);
    RunPayload {
        ast: ast_payload.ast,
        html: html_payload.html,
        diagnostics_summary: ast_payload.diagnostics_summary,
    }
}
