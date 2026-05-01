//! `marco-core` is a pure-Rust Markdown engine with editor intelligence.
//!
//! Typical flow:
//! 1. Parse input text into an AST with [`parse`].
//! 2. Render the AST to HTML with [`render()`].
//! 3. Optionally run editor analysis via [`MarkdownIntelligenceProvider`].
//!
//! # Example
//! ```rust
//! let doc = marco_core::parse("# Hello")?;
//! let html = marco_core::render(&doc, &marco_core::RenderOptions::default())?;
//! assert!(html.contains("<h1"));
//! assert!(html.contains("Hello"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Low-level Markdown grammar components (block and inline parsers).
pub mod grammar;
/// Editor intelligence APIs such as diagnostics, highlights, completions, and TOC.
pub mod intelligence;
/// Utility logic such as cache management and UTF-8 sanitization.
pub mod logic;
/// AST definitions and parser entry points.
pub mod parser;
/// HTML rendering and rendering options.
pub mod render;

/// Main intelligence provider used by downstream editor integrations.
pub use intelligence::MarkdownIntelligenceProvider;
/// Parse Markdown text into a [`Document`] AST.
pub use parser::parse;
/// Core AST types used by parser, renderer, and intelligence modules.
pub use parser::{Document, Node, NodeKind};
/// Render a parsed [`Document`] into HTML using [`RenderOptions`].
pub use render::{render, RenderOptions};

/// Convenience cache-backed parsing/rendering APIs and cache type.
pub use logic::cache::{parse_to_html, parse_to_html_cached, ParserCache};
/// UTF-8 sanitization API and related types.
pub use logic::utf8::{sanitize_input, sanitize_input_with_stats, InputSource, SanitizeStats};
