use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::cli::{Args, OutputMode};
use crate::logging::{append_log_event, LogEvent};
use crate::output::{run_ast_mode, run_both_mode, run_html_mode, run_intel_mode};

pub fn run(args: &Args, session_id: Option<uuid::Uuid>) -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let mut mode = args.mode.clone();
    let prompt = "marco-ast> ";

    println!("marco-ast interactive mode. Type :help for commands, :quit to exit.");
    println!("Current output mode: {mode}");

    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(trimmed);

                // Handle REPL commands
                if let Some(cmd) = trimmed.strip_prefix(':') {
                    match handle_command(cmd.trim(), &mut mode) {
                        CommandResult::Continue => continue,
                        CommandResult::Quit => break,
                        CommandResult::Unknown(c) => {
                            eprintln!("Unknown command: :{c}  (type :help for available commands)");
                            continue;
                        }
                    }
                }

                // Parse and process Markdown input
                let sanitized = marco_core::sanitize_input(trimmed.as_bytes(), marco_core::InputSource::Keyboard);
                match marco_core::parse(&sanitized) {
                    Ok(doc) => {
                        let payload = match &mode {
                            OutputMode::Ast => run_ast_mode(&doc, &sanitized, args),
                            OutputMode::Html => run_html_mode(&doc, &sanitized, args),
                            OutputMode::Both => run_both_mode(&doc, &sanitized, args),
                            OutputMode::Intel => run_intel_mode(&doc, &sanitized, args),
                        };

                        if args.log {
                            if let Some(sid) = session_id {
                                let event = LogEvent::build(
                                    sid,
                                    &mode,
                                    "interactive",
                                    trimmed,
                                    trimmed,
                                    &payload,
                                    None,
                                );
                                append_log_event(&event, &args.log_path)
                                    .unwrap_or_else(|err| eprintln!("warning: log write failed: {err}"));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("parse error: {e}");
                        if args.log {
                            if let Some(sid) = session_id {
                                let empty_payload = crate::output::RunPayload {
                                    ast: None,
                                    html: None,
                                    diagnostics_summary: None,
                                };
                                let event = LogEvent::build(
                                    sid,
                                    &mode,
                                    "interactive",
                                    trimmed,
                                    trimmed,
                                    &empty_payload,
                                    Some(e.to_string()),
                                );
                                append_log_event(&event, &args.log_path)
                                    .unwrap_or_else(|err| eprintln!("warning: log write failed: {err}"));
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: cancel current line, continue REPL
                println!("(interrupted)");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit
                break;
            }
            Err(e) => {
                eprintln!("readline error: {e}");
                break;
            }
        }
    }

    Ok(())
}

enum CommandResult {
    Continue,
    Quit,
    Unknown(String),
}

fn handle_command(cmd: &str, mode: &mut OutputMode) -> CommandResult {
    if cmd.starts_with("mode") {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        if let Some(m) = parts.get(1) {
            match *m {
                "ast" => {
                    *mode = OutputMode::Ast;
                    println!("Mode set to: ast");
                }
                "html" => {
                    *mode = OutputMode::Html;
                    println!("Mode set to: html");
                }
                "both" => {
                    *mode = OutputMode::Both;
                    println!("Mode set to: both");
                }
                "intel" => {
                    *mode = OutputMode::Intel;
                    println!("Mode set to: intel");
                }
                other => {
                    eprintln!("Unknown mode: {other}. Valid modes: ast, html, both, intel");
                }
            }
        } else {
            println!("Current mode: {mode}");
        }
        return CommandResult::Continue;
    }

    match cmd {
        "clear" => {
            print!("\x1b[2J\x1b[H");
            CommandResult::Continue
        }
        "help" => {
            println!("Commands:");
            println!("  :mode <ast|html|both|intel>  Switch output mode");
            println!("  :mode                        Show current mode");
            println!("  :clear                       Clear screen");
            println!("  :help                        Show this help");
            println!("  :quit / :exit                Exit");
            println!();
            println!("Enter any Markdown text to parse and display.");
            CommandResult::Continue
        }
        "quit" | "exit" => CommandResult::Quit,
        other => CommandResult::Unknown(other.to_string()),
    }
}
