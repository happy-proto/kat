mod analysis;
mod api;
mod armor_formats;
#[cfg(feature = "compact-parser-tables")]
mod compact_parser_tables;
mod debug_progress;
mod debug_runtime_log;
mod detection;
mod display_geometry;
mod document;
mod document_kind;
mod grammar_registry;
mod host_injections;
mod injection;
mod language_aliases;
mod language_runtime;
mod layout;
mod mise;
mod pkl;
mod projection;
mod proto_annotations;
mod python_docstrings;
mod render_data;
mod render_ops;
mod semantic_overlays;
mod source_highlight;
mod source_map;
mod span_ops;
mod sql_dialect;
mod terminal;
mod terminal_background;
mod theme;
mod timing;
mod visual;

pub use api::*;
pub use document::{
    DisplayRow, PreparedDocument, RenderOptions, RenderedDocument, RenderedRowAnchor,
};
pub use source_highlight::{HighlightBackgroundRun, HighlightSpan, HighlightStyle};
pub use timing::{RenderOutput, RenderTimings};

#[cfg(test)]
mod tests;
