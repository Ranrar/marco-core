use super::{EngineAdapter, EngineRun};
use crate::cli::Mode;
use comrak::{format_html, parse_document, Arena, Options};
use std::time::Instant;

pub struct ComrakAdapter;

impl EngineAdapter for ComrakAdapter {
    fn id(&self) -> &'static str {
        "comrak"
    }

    fn run_mode(&self, mode: Mode, input: &str) -> Result<EngineRun, String> {
        let opts = Options::default();

        let elapsed_ns = match mode {
            Mode::Parse => {
                let arena = Arena::new();
                let t = Instant::now();
                let _root = parse_document(&arena, input, &opts);
                t.elapsed().as_nanos()
            }
            Mode::Render => {
                // Pre-parse so timer covers only rendering
                let arena = Arena::new();
                let root = parse_document(&arena, input, &opts);
                let t = Instant::now();
                let mut html_out = String::with_capacity(input.len());
                format_html(root, &opts, &mut html_out)
                    .map_err(|e| format!("comrak render error: {e}"))?;
                t.elapsed().as_nanos()
            }
            Mode::E2e => {
                let arena = Arena::new();
                let t = Instant::now();
                let root = parse_document(&arena, input, &opts);
                let mut html_out = String::with_capacity(input.len());
                format_html(root, &opts, &mut html_out)
                    .map_err(|e| format!("comrak render error: {e}"))?;
                t.elapsed().as_nanos()
            }
            Mode::Intelligence => {
                return Err(
                    "comrak has no intelligence/LSP layer; mode 'intelligence' is unsupported"
                        .to_string(),
                );
            }
        };

        Ok(EngineRun {
            elapsed_ns,
            diagnostics_count: 0,
            highlights_count: 0,
        })
    }
}
