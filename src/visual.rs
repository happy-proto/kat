use crate::analysis::AnalysisDocument;
use crate::render_data::NestedRegion;
use crate::render_data::StyledSpan;
use crate::render_data::VisualRegion;
use crate::theme::ColorMode;
use crate::theme::TokenStyleSnapshot;
use serde::Serialize;

#[derive(Clone, Debug)]
pub(crate) struct VisualDocument {
    color_mode: ColorMode,
    spans: Vec<StyledSpan>,
    regions: Vec<VisualRegion>,
}

impl VisualDocument {
    pub(crate) fn from_parts(
        color_mode: ColorMode,
        spans: Vec<StyledSpan>,
        regions: Vec<VisualRegion>,
    ) -> Self {
        Self {
            color_mode,
            spans,
            regions,
        }
    }

    pub(crate) fn from_analysis(analysis: &AnalysisDocument) -> Self {
        Self::from_parts(
            analysis.theme().color_mode(),
            analysis.spans().to_vec(),
            collect_visual_regions(analysis.nested_regions()),
        )
    }

    pub(crate) fn spans(&self) -> &[StyledSpan] {
        &self.spans
    }

    pub(crate) fn regions(&self) -> &[VisualRegion] {
        &self.regions
    }

    pub(crate) fn snapshot(&self) -> VisualSnapshot {
        VisualSnapshot {
            color_mode: self.color_mode,
            spans: self
                .spans
                .iter()
                .map(|span| VisualSpanSnapshot {
                    start: span.range.start,
                    end: span.range.end,
                    style: span.style.map(|style| style.snapshot(self.color_mode)),
                })
                .collect(),
            regions: self
                .regions
                .iter()
                .map(|region| VisualRegionSnapshot {
                    visual_kind: region.visual_kind.snapshot_name(),
                    visual_level: region.visual_level,
                    segments: region
                        .segments
                        .iter()
                        .map(|segment| VisualRegionSegmentSnapshot {
                            line_start: segment.line_start,
                            left: segment.left,
                            text_end: segment.text_end,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VisualSnapshot {
    pub color_mode: ColorMode,
    pub spans: Vec<VisualSpanSnapshot>,
    pub regions: Vec<VisualRegionSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VisualSpanSnapshot {
    pub start: usize,
    pub end: usize,
    pub style: Option<TokenStyleSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VisualRegionSnapshot {
    pub visual_kind: &'static str,
    pub visual_level: usize,
    pub segments: Vec<VisualRegionSegmentSnapshot>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VisualRegionSegmentSnapshot {
    pub line_start: usize,
    pub left: usize,
    pub text_end: usize,
}

pub(crate) fn collect_visual_regions(nested_regions: &[NestedRegion]) -> Vec<VisualRegion> {
    let mut regions = Vec::new();

    for region in nested_regions {
        if region.visual_kind.is_block() {
            regions.push(VisualRegion {
                visual_kind: region.visual_kind,
                visual_level: region.visual_level,
                segments: region.layout_segments.clone(),
            });
        }
        regions.extend(region.child_regions.iter().cloned());
    }

    regions.sort_by_key(|region| {
        region
            .segments
            .first()
            .map(|segment| (segment.line_start, segment.left, region.visual_level))
            .unwrap_or((0, 0, region.visual_level))
    });
    regions
}
