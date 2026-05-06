mod ast_print;
mod cli;
mod input;
mod interactive;
mod logging;
mod output;

use clap::Parser;
use cli::{Args, OutputMode};
use output::TimingInfo;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let session_id = if args.log {
        Some(uuid::Uuid::new_v4())
    } else {
        None
    };

    if args.interactive {
        return interactive::run(&args, session_id);
    }

    let source = input::resolve_input(&args)?;
    let input_source = if args.file.is_some() {
        marco_core::InputSource::File
    } else {
        marco_core::InputSource::Keyboard
    };
    let sanitize_started = Instant::now();
    let (sanitized, sanitize_stats) =
        marco_core::sanitize_input_with_stats(source.as_bytes(), input_source);
    let sanitize_time = sanitize_started.elapsed();

    let parse_started = Instant::now();
    let doc = match marco_core::parse(&sanitized) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("parse error: {e}");
            if args.log {
                if let Some(sid) = session_id {
                    let empty_payload = output::RunPayload {
                        ast: None,
                        html: None,
                        diagnostics_summary: None,
                    };
                    let sk = source_kind(&args);
                    let sv = source_value(&args);
                    let event = logging::LogEvent::build(
                        sid,
                        &args.mode,
                        sk,
                        sv,
                        &source,
                        &empty_payload,
                        Some(e.to_string()),
                    );
                    logging::append_log_event(&event, &args.log_path)
                        .unwrap_or_else(|err| eprintln!("warning: log write failed: {err}"));
                }
            }
            std::process::exit(1);
        }
    };
    let timings = TimingInfo {
        sanitize: sanitize_time,
        parse: parse_started.elapsed(),
    };

    let payload = if args.json {
        output::run_json_mode(&doc, &sanitized, &sanitize_stats, &timings, &args.mode, &args)
    } else {
        match &args.mode {
            OutputMode::Ast => {
                output::run_ast_mode(&doc, &sanitized, &sanitize_stats, &timings, &args)
            }
            OutputMode::Html => {
                output::run_html_mode(&doc, &sanitized, &sanitize_stats, &timings, &args)
            }
            OutputMode::Both => {
                output::run_both_mode(&doc, &sanitized, &sanitize_stats, &timings, &args)
            }
            OutputMode::Intel => {
                output::run_intel_mode(&doc, &sanitized, &sanitize_stats, &timings, &args)
            }
        }
    };

    if args.log {
        if let Some(sid) = session_id {
            let event = logging::LogEvent::build(
                sid,
                &args.mode,
                source_kind(&args),
                source_value(&args),
                &source,
                &payload,
                None,
            );
            logging::append_log_event(&event, &args.log_path)
                .unwrap_or_else(|err| eprintln!("warning: log write failed: {err}"));
        }
    }

    Ok(())
}

fn source_kind(args: &Args) -> &'static str {
    if args.text.is_some() {
        "text"
    } else if args.file.is_some() {
        "file"
    } else {
        "stdin"
    }
}

fn source_value<'a>(args: &'a Args) -> &'a str {
    if let Some(t) = &args.text {
        t.as_str()
    } else if let Some(f) = &args.file {
        f.to_str().unwrap_or("<path>")
    } else {
        "<stdin>"
    }
}
