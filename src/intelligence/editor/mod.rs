//! Editor-facing intelligence features.

/// Markdown completion providers and completion item types.
pub mod completion;
/// Syntax highlight extraction from parsed Markdown AST.
pub mod highlight;
/// Hover information lookup utilities.
pub mod hover;

/// Re-export completion API.
pub use completion::{get_markdown_completions, CompletionItem};
/// Re-export highlight API.
pub use highlight::{compute_highlights, compute_highlights_with_source, Highlight, HighlightTag};
/// Re-export hover API.
pub use hover::{get_hover_info, get_position_span, HoverInfo};
