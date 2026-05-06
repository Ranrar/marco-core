use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "marco-ast",
    about = "Markdown AST viewer and renderer (marco-core)",
    version
)]
pub struct Args {
    /// Path to a Markdown file.
    pub file: Option<PathBuf>,

    /// Inline Markdown string.
    #[arg(short, long)]
    pub text: Option<String>,

    /// Interactive REPL mode.
    #[arg(short, long)]
    pub interactive: bool,

    /// Append all input/output events to a log file.
    #[arg(short, long)]
    pub log: bool,

    /// Log destination path [default: log.json].
    #[arg(long, default_value = "log.json")]
    pub log_path: PathBuf,

    /// Explicitly read from stdin pipe.
    #[arg(long)]
    pub stdin: bool,

    /// Output mode.
    #[arg(short, long, default_value = "both")]
    pub mode: OutputMode,

    /// Disable ANSI color output.
    #[arg(long)]
    pub no_color: bool,

    /// Compact AST (one line per node, no tree drawing).
    #[arg(long)]
    pub compact: bool,

    /// Emit a structured JSON report instead of human-readable sections.
    #[arg(long)]
    pub json: bool,

    /// Print line/column/byte spans next to AST nodes.
    #[arg(long)]
    pub spans: bool,

    /// Print source excerpts (byte-sliced from spans) next to AST nodes.
    #[arg(long)]
    pub excerpts: bool,

    /// Print UTF-8 / byte-vs-char inspection details.
    #[arg(long)]
    pub utf8: bool,

    /// Print sanitize/parse/render/intelligence timing summary.
    #[arg(long)]
    pub time: bool,

    /// Enable syntax highlighting in HTML output.
    #[arg(long)]
    pub syntax: bool,
}

#[derive(ValueEnum, Clone, PartialEq, Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Ast,
    Html,
    Both,
    Intel,
}

impl std::fmt::Display for OutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputMode::Ast => write!(f, "ast"),
            OutputMode::Html => write!(f, "html"),
            OutputMode::Both => write!(f, "both"),
            OutputMode::Intel => write!(f, "intel"),
        }
    }
}
