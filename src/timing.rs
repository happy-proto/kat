use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderTimings {
    pub detect_document_kind: Duration,
    pub highlight: Duration,
    pub semantic_overlays: Duration,
    pub injection_regions: Duration,
    pub nested_region_overlays: Duration,
    pub render_styled_spans: Duration,
}

impl RenderTimings {
    pub fn total_render_pipeline(&self) -> Duration {
        self.detect_document_kind
            + self.highlight
            + self.semantic_overlays
            + self.injection_regions
            + self.nested_region_overlays
            + self.render_styled_spans
    }

    pub(crate) fn record_detect(&mut self, started_at: Instant) {
        self.detect_document_kind += started_at.elapsed();
    }

    pub(crate) fn record_highlight(&mut self, started_at: Instant) {
        self.highlight += started_at.elapsed();
    }

    pub(crate) fn record_semantic_overlays(&mut self, started_at: Instant) {
        self.semantic_overlays += started_at.elapsed();
    }

    pub(crate) fn record_injection_regions(&mut self, started_at: Instant) {
        self.injection_regions += started_at.elapsed();
    }

    pub(crate) fn record_nested_region_overlays(&mut self, started_at: Instant) {
        self.nested_region_overlays += started_at.elapsed();
    }

    pub(crate) fn record_render_styled_spans(&mut self, started_at: Instant) {
        self.render_styled_spans += started_at.elapsed();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderOutput {
    pub output: String,
    pub timings: RenderTimings,
}
