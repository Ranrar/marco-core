use super::{EngineAdapter, EngineRun};
use crate::cli::Mode;
use std::time::Instant;

#[derive(Default)]
pub struct MarcoCoreAdapter;

impl EngineAdapter for MarcoCoreAdapter {
    fn id(&self) -> &'static str {
        "marco-core"
    }

    fn run_mode(&self, mode: Mode, input: &str) -> Result<EngineRun, String> {
        match mode {
            Mode::Parse => {
                let started = Instant::now();
                let _doc = marco_core::parse(input).map_err(|e| e.to_string())?;
                Ok(EngineRun {
                    elapsed_ns: started.elapsed().as_nanos(),
                    diagnostics_count: 0,
                    highlights_count: 0,
                })
            }
            Mode::Render => {
                let doc = marco_core::parse(input).map_err(|e| e.to_string())?;
                let started = Instant::now();
                let _html = marco_core::render(&doc, &marco_core::RenderOptions::default())
                    .map_err(|e| e.to_string())?;
                Ok(EngineRun {
                    elapsed_ns: started.elapsed().as_nanos(),
                    diagnostics_count: 0,
                    highlights_count: 0,
                })
            }
            Mode::E2e => {
                let started = Instant::now();
                let doc = marco_core::parse(input).map_err(|e| e.to_string())?;
                let _html = marco_core::render(&doc, &marco_core::RenderOptions::default())
                    .map_err(|e| e.to_string())?;
                Ok(EngineRun {
                    elapsed_ns: started.elapsed().as_nanos(),
                    diagnostics_count: 0,
                    highlights_count: 0,
                })
            }
            Mode::Intelligence => {
                let doc = marco_core::parse(input).map_err(|e| e.to_string())?;
                let started = Instant::now();
                let mut provider = marco_core::MarkdownIntelligenceProvider::new();
                provider.update_document(doc);
                let diagnostics = provider.diagnostics();
                let highlights = provider.highlights(input);
                Ok(EngineRun {
                    elapsed_ns: started.elapsed().as_nanos(),
                    diagnostics_count: diagnostics.len(),
                    highlights_count: highlights.len(),
                })
            }
        }
    }
}

/// Raw adapter: uses `parse_with_options` with all runtime cost-savers enabled
/// (no position tracking, no math parsing, no diagram parsing).
/// Used to measure the minimum parser hot-path cost.
#[derive(Default)]
pub struct MarcoCoreRawAdapter;

impl EngineAdapter for MarcoCoreRawAdapter {
    fn id(&self) -> &'static str {
        "marco-core-raw"
    }

    fn run_mode(&self, mode: Mode, input: &str) -> Result<EngineRun, String> {
        let raw_opts = marco_core::ParseOptions {
            track_positions: false,
            parse_math: false,
            parse_diagrams: false,
        };

        match mode {
            Mode::Parse => {
                let started = Instant::now();
                let _doc = marco_core::parse_with_options(input, raw_opts)
                    .map_err(|e| e.to_string())?;
                Ok(EngineRun {
                    elapsed_ns: started.elapsed().as_nanos(),
                    diagnostics_count: 0,
                    highlights_count: 0,
                })
            }
            Mode::Render => {
                let doc = marco_core::parse_with_options(input, raw_opts)
                    .map_err(|e| e.to_string())?;
                let started = Instant::now();
                let _html = marco_core::render(&doc, &marco_core::RenderOptions::default())
                    .map_err(|e| e.to_string())?;
                Ok(EngineRun {
                    elapsed_ns: started.elapsed().as_nanos(),
                    diagnostics_count: 0,
                    highlights_count: 0,
                })
            }
            Mode::E2e => {
                let started = Instant::now();
                let doc = marco_core::parse_with_options(input, raw_opts)
                    .map_err(|e| e.to_string())?;
                let _html = marco_core::render(&doc, &marco_core::RenderOptions::default())
                    .map_err(|e| e.to_string())?;
                Ok(EngineRun {
                    elapsed_ns: started.elapsed().as_nanos(),
                    diagnostics_count: 0,
                    highlights_count: 0,
                })
            }
            // Intelligence not meaningful without position tracking
            Mode::Intelligence => Err(
                "marco-core-raw does not support intelligence mode (track_positions=false)"
                    .to_string(),
            ),
        }
    }
}
