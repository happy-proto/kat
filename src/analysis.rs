use std::{path::Path, time::Instant};

use anyhow::Result;
use serde::Serialize;

use crate::{
    HighlightRenderData, NestedRegion, RenderTimings, StyledSpan, detect_document_kind,
    document_kind::{DocumentKind, DocumentKindSnapshot},
    named_document_kind,
    theme::{ColorMode, Theme, TokenStyleSnapshot},
};

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
        let render_data = crate::highlight_named_language_render_data(
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
    pub right_padding: usize,
}

impl From<crate::RegionSegment> for RegionSegmentSnapshot {
    fn from(segment: crate::RegionSegment) -> Self {
        Self {
            line_start: segment.line_start,
            left: segment.left,
            text_end: segment.text_end,
            left_column_override: segment.left_column_override.map(|column| column.as_usize()),
            right_padding: segment.right_padding.as_usize(),
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
    fn from_region(region: &crate::VisualRegion) -> Self {
        Self {
            visual_kind: region.visual_kind.snapshot_name(),
            visual_level: region.visual_level,
            segments: region.segments.iter().copied().map(Into::into).collect(),
        }
    }
}
