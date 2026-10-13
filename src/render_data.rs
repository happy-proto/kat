use crate::display_geometry::DisplayColumn;
use crate::document_kind::DocumentKind;
use crate::host_injections::InjectionVisualKind;
use crate::theme::TokenStyle;
use std::ops::Range;

pub(crate) struct HighlightRenderData {
    pub(crate) resolved_document_kind: DocumentKind,
    pub(crate) spans: Vec<StyledSpan>,
    pub(crate) nested_regions: Vec<NestedRegion>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VisualRegion {
    pub(crate) visual_kind: InjectionVisualKind,
    pub(crate) visual_level: usize,
    pub(crate) segments: Vec<RegionSegment>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionSegment {
    pub(crate) line_start: usize,
    pub(crate) left: usize,
    pub(crate) text_end: usize,
    pub(crate) left_column_override: Option<DisplayColumn>,
}

#[derive(Clone, Debug)]
pub(crate) struct NestedRegion {
    pub(crate) visual_level: usize,
    pub(crate) resolved_document_kind: DocumentKind,
    pub(crate) visual_kind: InjectionVisualKind,
    pub(crate) layout_segments: Vec<RegionSegment>,
    pub(crate) child_regions: Vec<VisualRegion>,
    pub(crate) child_nested_regions: Vec<NestedRegion>,
    pub(crate) overlays: Vec<StyledSpan>,
    pub(crate) merge_parent_styles: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StyledSpan {
    pub(crate) range: Range<usize>,
    pub(crate) style: Option<TokenStyle>,
    pub(crate) hyperlink: Option<String>,
}
