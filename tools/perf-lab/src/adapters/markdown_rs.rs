use super::{EngineAdapter, EngineRun};
use crate::cli::Mode;

pub struct MarkdownRsAdapter;

impl EngineAdapter for MarkdownRsAdapter {
    fn id(&self) -> &'static str {
        "markdown-rs"
    }

    fn run_mode(&self, mode: Mode, _input: &str) -> Result<EngineRun, String> {
        Err(format!(
            "adapter '{}' is not implemented yet for mode '{:?}'",
            self.id(),
            mode
        ))
    }
}
