use super::{EngineAdapter, EngineRun};
use crate::cli::Mode;
use pulldown_cmark::{html, Options, Parser};
use std::time::Instant;

pub struct PulldownCmarkAdapter;

impl EngineAdapter for PulldownCmarkAdapter {
    fn id(&self) -> &'static str {
        "pulldown-cmark"
    }

    fn run_mode(&self, mode: Mode, input: &str) -> Result<EngineRun, String> {
        let opts = Options::all();

        let elapsed_ns = match mode {
            Mode::Parse => {
                let t = Instant::now();
                // Consume all events — pulling is what costs time
                let _count = Parser::new_ext(input, opts).count();
                t.elapsed().as_nanos()
            }
            Mode::Render => {
                // Pre-parse so timer covers only rendering
                let events: Vec<_> = Parser::new_ext(input, opts).collect();
                let t = Instant::now();
                let mut html_out = String::with_capacity(input.len());
                html::push_html(&mut html_out, events.into_iter());
                t.elapsed().as_nanos()
            }
            Mode::E2e => {
                let t = Instant::now();
                let events = Parser::new_ext(input, opts);
                let mut html_out = String::with_capacity(input.len());
                html::push_html(&mut html_out, events);
                t.elapsed().as_nanos()
            }
            Mode::Intelligence => {
                return Err(
                    "pulldown-cmark has no intelligence/LSP layer; mode 'intelligence' is unsupported"
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
