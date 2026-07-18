//! Shared primitives for deferring inline parsing so it can be fanned out
//! across cores with rayon (the `parallel-parse` feature).
//!
//! Three call sites in this module tree defer inline parsing instead of
//! doing it eagerly: paragraphs + footnote-definition bodies (`mod.rs`'s
//! main loop), table cells (`gfm_table_parser.rs`), and definition-list
//! terms (`mod.rs::parse_extended_definition_list`). Each collects its own
//! flat list of not-yet-inline-parsed content, in whatever shape it already
//! naturally builds, and calls [`resolve_pending_batch`] once before
//! returning its own `Vec<Node>` to its caller — so nested containers
//! (blockquotes, lists, ...) get their own independent fan-out for free,
//! with no change to the (unrelated, untouched) recursive block-scanning
//! that builds them.
//!
//! Deliberately *not* a new `NodeKind` variant: the pending list lives in a
//! side `Vec`, local to whichever function is about to finalize its own
//! `Vec<Node>`, so the public AST shape (`Node`/`NodeKind`) stays untouched.
//! Reattachment is done by *position*, not by node identity/address —
//! unlike `parallel-render`'s `precompute_highlights`, the AST here is
//! still being built (nodes pushed into growing `Vec`s, whole `Vec<Node>`s
//! later moved into parent containers) while this runs, so addresses
//! captured mid-scan would go stale. Position-based zipping sidesteps that
//! entirely: rayon's `collect()` into a `Vec` preserves the original order
//! of an `IndexedParallelIterator`, so results line up with the request
//! list without needing a key at all.

use super::shared::GrammarSpan;
use crate::parser::ast::{Node, NodeKind};
use crate::parser::shared::{
    opt_span, parse_diagrams_enabled, parse_math_enabled, track_positions_enabled,
    ParseOptionsGuard,
};
use rayon::prelude::*;

/// Raw content awaiting inline parsing, deferred so it can be resolved in a
/// batch. Most call sites use `Borrowed` (a span into whatever input the
/// current recursion level owns); footnote-definition bodies are already
/// hand-assembled, non-contiguous owned strings today (continuation lines
/// have their indentation stripped, so the content isn't a contiguous
/// substring of anything), so `Owned` exists to carry those without forcing
/// an artificial extra allocation/copy anywhere else.
pub(crate) enum PendingSpan<'a> {
    Borrowed(GrammarSpan<'a>),
    Owned(String),
}

/// One piece of a paragraph's (or footnote-definition body's) content: text
/// that still needs inline parsing, or content that's already a finished
/// node (an embedded `TaskCheckboxInline` marker, which the paragraph
/// parser builds directly rather than via the inline parser).
pub(crate) enum Segment<'a> {
    Literal(Node),
    Pending(PendingSpan<'a>),
}

/// A paragraph-shaped node awaiting its `children`, recorded during a block
/// scan so it can be patched in after a batched, parallel resolve.
///
/// `node_index` locates the node within whichever flat `Vec<Node>` is being
/// built by the current call (stable because that `Vec` is only ever
/// appended to, never reordered, between recording this and patching it).
/// `nested_child_index` distinguishes two shapes: a plain paragraph's own
/// `children` (`None`), or — for a footnote-definition node, which wraps a
/// single childless `Paragraph` — that inner paragraph's `children`
/// (`Some(0)`), since it's `nodes[node_index]` itself that must stay a
/// `FootnoteDefinition` with one `Paragraph` child, not be overwritten.
pub(crate) struct PendingLeaf<'a> {
    pub(crate) node_index: usize,
    pub(crate) nested_child_index: Option<usize>,
    pub(crate) segments: Vec<Segment<'a>>,
}

/// Resolve every leaf's segments in one shared parallel batch and patch the
/// results directly into `nodes` (see [`PendingLeaf`] for where each result
/// goes).
pub(crate) fn apply_pending_leaves(nodes: &mut [Node], leaves: Vec<PendingLeaf<'_>>) {
    if leaves.is_empty() {
        return;
    }

    let mut targets: Vec<(usize, Option<usize>)> = Vec::with_capacity(leaves.len());
    let mut segment_lists: Vec<Vec<Segment<'_>>> = Vec::with_capacity(leaves.len());
    for leaf in leaves {
        targets.push((leaf.node_index, leaf.nested_child_index));
        segment_lists.push(leaf.segments);
    }

    let resolved = resolve_segmented_batch(segment_lists);

    for ((node_index, nested_child_index), children) in targets.into_iter().zip(resolved) {
        match nested_child_index {
            None => nodes[node_index].children = children,
            Some(child_index) => nodes[node_index].children[child_index].children = children,
        }
    }
}

/// Below this many items, resolve sequentially instead of dispatching
/// through rayon.
///
/// Each container nesting level (list item, blockquote, ...) gets its own
/// independent batch (see module docs) — a list of many items, each
/// containing one short paragraph, produces many single-item batches, not
/// one large one. Measured directly on `fixture:large/generated-synthetic.md`
/// (a 500-item list, ~37 KB): with no threshold, `parse` under
/// `--features parallel-parse` was ~28% *slower* than the sequential
/// default (rayon's per-`par_iter()`-call dispatch overhead, paid ~500
/// times for batches with nothing worth parallelizing, dominated any
/// benefit). This threshold exists specifically to avoid that regression on
/// exactly the kind of document this crate is expected to handle well.
const PARALLEL_THRESHOLD: usize = 4;

/// Resolve a batch of pending spans into their parsed inline children,
/// preserving the original order (index `i` of the result corresponds to
/// index `i` of `items`). Dispatches through rayon only when the batch is
/// large enough for that to be worth its overhead (see
/// [`PARALLEL_THRESHOLD`]); smaller batches resolve directly on the calling
/// thread.
///
/// Captures the calling thread's current `ParseOptions`
/// (`track_positions`/`parse_math`/`parse_diagrams`) *before* dispatching,
/// then re-installs them via `ParseOptionsGuard` inside every parallel
/// closure. This is required, not cosmetic: those options are thread-locals
/// set once on the calling thread by `parse_with_options`
/// (`src/parser/mod.rs`) — a rayon worker thread that never ran that guard
/// would otherwise see the compiled-in defaults (`true`/`true`/`true`)
/// regardless of what the caller actually asked for, silently computing
/// spans (or parsing math/diagrams) the caller explicitly opted out of. The
/// below-threshold path needs no such guard: it runs entirely on the
/// calling thread, which already has the correct values installed.
pub(crate) fn resolve_pending_batch(items: Vec<PendingSpan<'_>>) -> Vec<Vec<Node>> {
    if items.len() < PARALLEL_THRESHOLD {
        return items.iter().map(resolve_one).collect();
    }

    let track = track_positions_enabled();
    let math = parse_math_enabled();
    let diagrams = parse_diagrams_enabled();

    items
        .par_iter()
        .map(|item| {
            let _guard = ParseOptionsGuard::new(track, math, diagrams);
            resolve_one(item)
        })
        .collect()
}

fn resolve_one(item: &PendingSpan<'_>) -> Vec<Node> {
    match item {
        PendingSpan::Borrowed(span) => {
            match crate::parser::inlines::parse_inlines_from_span(*span) {
                Ok(children) => children,
                Err(e) => {
                    log::warn!("Failed to parse inline elements: {}", e);
                    vec![Node {
                        kind: NodeKind::Text(span.fragment().to_string()),
                        span: opt_span(*span),
                        children: Vec::new(),
                    }]
                }
            }
        }
        PendingSpan::Owned(text) => match crate::parser::inlines::parse_inlines(text) {
            Ok(children) => children,
            Err(_) => vec![Node {
                kind: NodeKind::Text(text.clone()),
                span: None,
                children: Vec::new(),
            }],
        },
    }
}

/// Flatten a list of paragraphs' segments into their `Vec<Node>` children,
/// resolving every `Segment::Pending` in one shared parallel batch across
/// *all* paragraphs passed in, and splicing `Segment::Literal` nodes back
/// into their original positions.
///
/// `paragraphs` is a list of segment-lists (one per paragraph or
/// footnote-definition body); the return value has the same length and
/// per-paragraph order, ready to assign directly to each `Node.children`.
pub(crate) fn resolve_segmented_batch(paragraphs: Vec<Vec<Segment<'_>>>) -> Vec<Vec<Node>> {
    // A slot is either a finished literal node (passed through untouched)
    // or an explicit marker to splice in the next resolved-batch result.
    // Using an explicit marker (rather than inferring "this must be a
    // placeholder" from node content) avoids any chance of colliding with a
    // real, legitimately-empty literal node.
    enum Slot {
        Literal(Node),
        Resolved,
    }

    // Single pass: consume `paragraphs`, pulling every Pending span out
    // (in order, across all paragraphs) into one flat batch, while
    // recording each paragraph's own shape as a same-length list of slots.
    let mut pending_spans: Vec<PendingSpan<'_>> = Vec::new();
    let plans: Vec<Vec<Slot>> = paragraphs
        .into_iter()
        .map(|segments| {
            segments
                .into_iter()
                .map(|segment| match segment {
                    Segment::Literal(node) => Slot::Literal(node),
                    Segment::Pending(span) => {
                        pending_spans.push(span);
                        Slot::Resolved
                    }
                })
                .collect()
        })
        .collect();

    let mut resolved = resolve_pending_batch(pending_spans).into_iter();

    plans
        .into_iter()
        .map(|slots| {
            let mut children = Vec::new();
            for slot in slots {
                match slot {
                    Slot::Literal(node) => children.push(node),
                    Slot::Resolved => children.extend(resolved.next().unwrap_or_default()),
                }
            }
            children
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `resolve_segmented_batch`/`resolve_pending_batch`'s entire zip-back
    /// design relies on rayon's `collect()` preserving the original order
    /// of an `IndexedParallelIterator` (documented rayon behavior, but
    /// foundational enough here to verify directly rather than only cite).
    #[test]
    fn rayon_par_iter_collect_preserves_order() {
        let input: Vec<usize> = (0..500).collect();
        let doubled: Vec<usize> = input.par_iter().map(|n| n * 2).collect();
        let expected: Vec<usize> = input.iter().map(|n| n * 2).collect();
        assert_eq!(doubled, expected);
    }

    #[test]
    fn resolve_pending_batch_preserves_order_and_content() {
        let texts = ["alpha", "beta", "gamma", "delta", "epsilon"];
        let items: Vec<PendingSpan> = texts
            .iter()
            .map(|t| PendingSpan::Owned(t.to_string()))
            .collect();

        let results = resolve_pending_batch(items);
        assert_eq!(results.len(), texts.len());
        for (result, text) in results.iter().zip(texts.iter()) {
            let NodeKind::Text(got) = &result[0].kind else {
                panic!("expected a single Text node for plain input {text:?}");
            };
            assert_eq!(got, text);
        }
    }

    #[test]
    fn resolve_segmented_batch_interleaves_literals_and_pending() {
        let literal = |label: &str| Node {
            kind: NodeKind::Text(label.to_string()),
            span: None,
            children: Vec::new(),
        };

        // Paragraph 1: literal, pending, literal.
        // Paragraph 2: pending only.
        let paragraphs = vec![
            vec![
                Segment::Literal(literal("before")),
                Segment::Pending(PendingSpan::Owned("middle".to_string())),
                Segment::Literal(literal("after")),
            ],
            vec![Segment::Pending(PendingSpan::Owned("solo".to_string()))],
        ];

        let resolved = resolve_segmented_batch(paragraphs);
        assert_eq!(resolved.len(), 2);

        // Paragraph 1: literal "before" untouched, then the resolved
        // "middle" text node(s), then literal "after" untouched.
        assert!(matches!(&resolved[0][0].kind, NodeKind::Text(t) if t == "before"));
        assert!(matches!(&resolved[0].last().unwrap().kind, NodeKind::Text(t) if t == "after"));
        let middle_text: String = resolved[0][1..resolved[0].len() - 1]
            .iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(middle_text, "middle");

        // Paragraph 2: just the resolved "solo" text.
        assert!(matches!(&resolved[1][0].kind, NodeKind::Text(t) if t == "solo"));
    }
}
