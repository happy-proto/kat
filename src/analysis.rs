use crate::armor_formats::render_armor_format;
use crate::debug_progress::log as progress_log;
use crate::detection::detect_document_kind;
use crate::detection::resolve_highlight_document_kind;
use crate::document_kind::DocumentKind;
use crate::document_kind::DocumentKindSnapshot;
use crate::document_kind::named_document_kind;
use crate::injection::collect_top_level_injection_regions_from_tree;
use crate::language_runtime::global_highlight_name;
use crate::language_runtime::runtime;
use crate::render_data::HighlightRenderData;
use crate::render_data::NestedRegion;
use crate::render_data::StyledSpan;
use crate::semantic_overlays::parse_language_tree;
use crate::semantic_overlays::semantic_capture_spans_with_tree;
use crate::span_ops::overlay_nested_region;
use crate::span_ops::overlay_style_spans;
use crate::span_ops::push_span;
use crate::span_ops::style_covering_span;
use crate::theme::ColorMode;
use crate::theme::Theme;
use crate::theme::TokenStyleSnapshot;
use crate::timing::RenderTimings;
use anyhow::Context;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use std::time::Instant;
use tree_sitter::Tree;
use tree_sitter_highlight::Highlight;
use tree_sitter_highlight::HighlightEvent;
use tree_sitter_highlight::Highlighter;

#[derive(Clone, Debug)]
pub(crate) struct AnalysisDocument {
    theme: Theme,
    detected_document_kind: Option<DocumentKind>,
    resolved_document_kind: Option<DocumentKind>,
    spans: Vec<StyledSpan>,
    nested_regions: Vec<NestedRegion>,
}

impl AnalysisDocument {
    pub(crate) fn document_kind(
        document_kind: DocumentKind,
        source_path: Option<&Path>,
        source: &str,
        theme: Theme,
        timings: Option<&mut RenderTimings>,
    ) -> Result<Self> {
        let render_data = crate::analysis::highlight_named_language_render_data(
            document_kind,
            source_path,
            source,
            &theme,
            timings,
        )?;
        Ok(Self::from_render_data(
            theme,
            Some(document_kind),
            render_data,
        ))
    }

    pub(crate) fn detect(
        source_path: Option<&Path>,
        source: &str,
        theme: Theme,
        mut timings: Option<&mut RenderTimings>,
    ) -> Result<Self> {
        let detect_started_at = Instant::now();
        let detected_document_kind = detect_document_kind(source_path, source);
        if let Some(timings) = timings.as_deref_mut() {
            timings.record_detect(detect_started_at);
        }
        let Some(document_kind) = detected_document_kind else {
            return Ok(Self {
                theme,
                detected_document_kind: None,
                resolved_document_kind: None,
                spans: Vec::new(),
                nested_regions: Vec::new(),
            });
        };

        Self::document_kind(document_kind, source_path, source, theme, timings)
    }

    pub(crate) fn named_language(
        language_name: &str,
        source: &str,
        theme: Theme,
        timings: Option<&mut RenderTimings>,
    ) -> Result<Self> {
        Self::document_kind(
            named_document_kind(language_name)?,
            None,
            source,
            theme,
            timings,
        )
    }

    fn from_render_data(
        theme: Theme,
        detected_document_kind: Option<DocumentKind>,
        render_data: HighlightRenderData,
    ) -> Self {
        Self {
            theme,
            detected_document_kind,
            resolved_document_kind: Some(render_data.resolved_document_kind),
            spans: render_data.spans,
            nested_regions: render_data.nested_regions,
        }
    }

    pub(crate) fn theme(&self) -> Theme {
        self.theme
    }

    pub(crate) fn spans(&self) -> &[StyledSpan] {
        &self.spans
    }

    pub(crate) fn nested_regions(&self) -> &[NestedRegion] {
        &self.nested_regions
    }

    pub(crate) fn is_plain_passthrough(&self) -> bool {
        self.detected_document_kind.is_none()
    }

    pub(crate) fn snapshot(&self) -> AnalysisSnapshot {
        AnalysisSnapshot {
            color_mode: self.theme.color_mode(),
            detected_document_kind: self.detected_document_kind.map(DocumentKind::snapshot),
            resolved_document_kind: self.resolved_document_kind.map(DocumentKind::snapshot),
            spans: self
                .spans
                .iter()
                .map(|span| StyledSpanSnapshot::from_span(span, self.theme.color_mode()))
                .collect(),
            nested_regions: self
                .nested_regions
                .iter()
                .map(|region| NestedRegionSnapshot::from_region(region, self.theme.color_mode()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct AnalysisSnapshot {
    pub color_mode: ColorMode,
    pub detected_document_kind: Option<DocumentKindSnapshot>,
    pub resolved_document_kind: Option<DocumentKindSnapshot>,
    pub spans: Vec<StyledSpanSnapshot>,
    pub nested_regions: Vec<NestedRegionSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct StyledSpanSnapshot {
    pub start: usize,
    pub end: usize,
    pub style: Option<TokenStyleSnapshot>,
}

impl StyledSpanSnapshot {
    fn from_span(span: &StyledSpan, color_mode: ColorMode) -> Self {
        Self {
            start: span.range.start,
            end: span.range.end,
            style: span.style.map(|style| style.snapshot(color_mode)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NestedRegionSnapshot {
    pub visual_level: usize,
    pub resolved_document_kind: DocumentKindSnapshot,
    pub visual_kind: &'static str,
    pub merge_parent_styles: bool,
    pub layout_segments: Vec<RegionSegmentSnapshot>,
    pub overlays: Vec<StyledSpanSnapshot>,
    pub child_regions: Vec<VisualRegionSnapshot>,
    pub child_nested_regions: Vec<NestedRegionSnapshot>,
}

impl NestedRegionSnapshot {
    fn from_region(region: &NestedRegion, color_mode: ColorMode) -> Self {
        Self {
            visual_level: region.visual_level,
            resolved_document_kind: region.resolved_document_kind.snapshot(),
            visual_kind: region.visual_kind.snapshot_name(),
            merge_parent_styles: region.merge_parent_styles,
            layout_segments: region
                .layout_segments
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
            overlays: region
                .overlays
                .iter()
                .map(|span| StyledSpanSnapshot::from_span(span, color_mode))
                .collect(),
            child_regions: region
                .child_regions
                .iter()
                .map(VisualRegionSnapshot::from_region)
                .collect(),
            child_nested_regions: region
                .child_nested_regions
                .iter()
                .map(|region| NestedRegionSnapshot::from_region(region, color_mode))
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RegionSegmentSnapshot {
    pub line_start: usize,
    pub left: usize,
    pub text_end: usize,
    pub left_column_override: Option<usize>,
}

impl From<crate::render_data::RegionSegment> for RegionSegmentSnapshot {
    fn from(segment: crate::render_data::RegionSegment) -> Self {
        Self {
            line_start: segment.line_start,
            left: segment.left,
            text_end: segment.text_end,
            left_column_override: segment.left_column_override.map(|column| column.as_usize()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VisualRegionSnapshot {
    pub visual_kind: &'static str,
    pub visual_level: usize,
    pub segments: Vec<RegionSegmentSnapshot>,
}

impl VisualRegionSnapshot {
    fn from_region(region: &crate::render_data::VisualRegion) -> Self {
        Self {
            visual_kind: region.visual_kind.snapshot_name(),
            visual_level: region.visual_level,
            segments: region.segments.iter().copied().map(Into::into).collect(),
        }
    }
}

pub(crate) fn highlight_named_language_render_data(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
    timings: Option<&mut RenderTimings>,
) -> Result<HighlightRenderData> {
    highlight_named_language_render_data_with_depth(
        document_kind,
        source_path,
        source,
        theme,
        0,
        timings,
    )
}

pub(crate) fn highlight_named_language_render_data_with_depth(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
    nested_depth: usize,
    mut timings: Option<&mut RenderTimings>,
) -> Result<HighlightRenderData> {
    let resolved_document_kind =
        resolve_highlight_document_kind(document_kind, source_path, source);
    let resolved_runtime_name = resolved_document_kind.runtime_name();
    if matches!(resolved_runtime_name, "pem" | "openpgp_armor") {
        return render_armor_format(resolved_document_kind, source, theme);
    }
    let log_progress = source_path.is_some();
    if log_progress {
        progress_log(
            "highlight",
            format!(
                "begin runtime={} depth={} bytes={}",
                resolved_runtime_name,
                nested_depth,
                source.len()
            ),
        );
    }
    let language_runtime = runtime(resolved_runtime_name)
        .with_context(|| format!("missing language runtime for {resolved_runtime_name}"))?;
    let highlight_started_at = Instant::now();
    let mut highlighter = Highlighter::new();
    let events = highlighter
        .highlight(
            &language_runtime.flat_configuration,
            source.as_bytes(),
            None,
            None,
            |_| None,
        )
        .context("failed to highlight source")?;

    let mut spans = collect_styled_spans(source, events, theme)?;
    if log_progress {
        progress_log(
            "highlight",
            format!(
                "highlight_done runtime={} spans={} elapsed={:.3}ms",
                resolved_runtime_name,
                spans.len(),
                highlight_started_at.elapsed().as_secs_f64() * 1_000.0
            ),
        );
    }
    if let Some(timings) = timings.as_deref_mut() {
        timings.record_highlight(highlight_started_at);
    }

    let semantic_started_at = Instant::now();
    // The highlighter owns its tree internally. Share our analysis tree between
    // semantic overlays and injections rather than parsing each phase again.
    let tree = parse_language_tree(resolved_runtime_name, source)?;
    spans = overlay_semantic_captures(resolved_document_kind, source, &tree, spans, theme)?;
    if log_progress {
        progress_log(
            "semantic",
            format!(
                "done runtime={} spans={} elapsed={:.3}ms",
                resolved_runtime_name,
                spans.len(),
                semantic_started_at.elapsed().as_secs_f64() * 1_000.0
            ),
        );
    }
    if let Some(timings) = timings.as_deref_mut() {
        timings.record_semantic_overlays(semantic_started_at);
    }

    let injections_started_at = Instant::now();
    let nested_regions = collect_top_level_injection_regions_from_tree(
        resolved_document_kind,
        source,
        &tree,
        theme,
        nested_depth,
        timings.as_deref_mut(),
    )?;
    if log_progress {
        progress_log(
            "injections",
            format!(
                "regions_done runtime={} regions={} elapsed={:.3}ms",
                resolved_runtime_name,
                nested_regions.len(),
                injections_started_at.elapsed().as_secs_f64() * 1_000.0
            ),
        );
    }
    if let Some(timings) = timings.as_deref_mut() {
        timings.record_injection_regions(injections_started_at);
    }

    let nested_overlay_started_at = Instant::now();
    for region in &nested_regions {
        spans = overlay_nested_region(spans, region);
    }
    if log_progress {
        progress_log(
            "nested_overlays",
            format!(
                "done runtime={} spans={} regions={} elapsed={:.3}ms",
                resolved_runtime_name,
                spans.len(),
                nested_regions.len(),
                nested_overlay_started_at.elapsed().as_secs_f64() * 1_000.0
            ),
        );
    }
    if let Some(timings) = timings {
        timings.record_nested_region_overlays(nested_overlay_started_at);
    }

    Ok(HighlightRenderData {
        resolved_document_kind,
        spans,
        nested_regions,
    })
}

pub(crate) fn overlay_semantic_captures(
    document_kind: DocumentKind,
    source: &str,
    tree: &Tree,
    parent_spans: Vec<StyledSpan>,
    theme: &Theme,
) -> Result<Vec<StyledSpan>> {
    let invalid_style = theme.token_style_for("invalid.illegal.regex", "");
    let overlays = semantic_capture_spans_with_tree(document_kind, source, tree)?
        .into_iter()
        .filter_map(|span| {
            let text = &source[span.range.clone()];
            theme
                .token_style_for(span.capture, text)
                .map(|overlay_style| {
                    let style = match (
                        style_covering_span(&parent_spans, span.range.start, span.range.end),
                        invalid_style,
                    ) {
                        (Some(parent_style), Some(invalid_style))
                            if parent_style == invalid_style =>
                        {
                            Some(parent_style.merge(overlay_style))
                        }
                        _ => Some(overlay_style),
                    };
                    StyledSpan {
                        range: span.range,
                        style,
                        hyperlink: None,
                    }
                })
        })
        .collect::<Vec<_>>();

    Ok(overlay_style_spans(parent_spans, overlays))
}

pub(crate) fn collect_styled_spans<'a>(
    source: &str,
    events: impl Iterator<Item = Result<HighlightEvent, tree_sitter_highlight::Error>> + 'a,
    theme: &Theme,
) -> Result<Vec<StyledSpan>> {
    let mut active_highlights: Vec<Highlight> = Vec::new();
    let mut spans = Vec::new();

    for event in events {
        match event.context("failed to process highlight event")? {
            HighlightEvent::Source { start, end } => {
                if start == end {
                    continue;
                }

                let text = &source[start..end];
                let active_captures = active_highlights
                    .iter()
                    .map(|highlight| global_highlight_name(highlight.0))
                    .collect::<Vec<_>>();
                let style = theme
                    .merged_token_style_for(active_captures.iter().copied(), text)
                    .or_else(|| theme.default_style());
                let hyperlink = active_captures
                    .contains(&"text.uri")
                    .then(|| crate::terminal::terminal_hyperlink_destination(text))
                    .flatten();
                push_span(&mut spans, start..end, style, hyperlink);
            }
            HighlightEvent::HighlightStart(highlight) => active_highlights.push(highlight),
            HighlightEvent::HighlightEnd => {
                active_highlights.pop();
            }
        }
    }

    if spans.is_empty() && !source.is_empty() {
        spans.push(StyledSpan {
            range: 0..source.len(),
            style: theme.default_style(),
            hyperlink: None,
        });
    }

    Ok(spans)
}
