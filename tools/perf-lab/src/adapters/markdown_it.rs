use super::{EngineAdapter, EngineRun};
use crate::cli::Mode;

pub struct MarkdownItRsAdapter;

impl EngineAdapter for MarkdownItRsAdapter {
    fn id(&self) -> &'static str {
        "markdown-it-rs"
    }

    fn run_mode(&self, mode: Mode, _input: &str) -> Result<EngineRun, String> {
        Err(format!(
            "adapter '{}' is not implemented yet for mode '{:?}'",
            self.id(),
            mode
        ))
    }
}
