use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderTimings {
    pub detect_document_kind: Duration,
    pub highlight: Duration,
    pub analysis_parse: Duration,
    pub semantic_overlays: Duration,
    pub injection_regions: Duration,
    pub nested_region_overlays: Duration,
    pub layout: Duration,
    pub render_styled_spans: Duration,
}

impl RenderTimings {
    /// Sum of exclusive measured stages, including nested documents once.
    pub fn total_render_pipeline(&self) -> Duration {
        self.detect_document_kind
            + self.highlight
            + self.analysis_parse
            + self.semantic_overlays
            + self.injection_regions
            + self.nested_region_overlays
            + self.layout
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

    pub(crate) fn record_injections(&mut self, elapsed: Duration, children: &Self) {
        self.injection_regions += elapsed.saturating_sub(children.total_render_pipeline());
        self.merge(children);
    }

    /// Accumulate measurements from another document without changing their meaning.
    pub fn merge(&mut self, other: &Self) {
        self.detect_document_kind += other.detect_document_kind;
        self.highlight += other.highlight;
        self.analysis_parse += other.analysis_parse;
        self.semantic_overlays += other.semantic_overlays;
        self.injection_regions += other.injection_regions;
        self.nested_region_overlays += other.nested_region_overlays;
        self.layout += other.layout;
        self.render_styled_spans += other.render_styled_spans;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn injection_scope_is_exclusive_of_children() {
        let children = RenderTimings {
            highlight: Duration::from_millis(5),
            injection_regions: Duration::from_millis(3),
            ..Default::default()
        };
        let mut parent = RenderTimings::default();
        parent.record_injections(Duration::from_millis(10), &children);
        assert_eq!(parent.total_render_pipeline(), Duration::from_millis(10));
        assert_eq!(parent.highlight, children.highlight);
        assert_eq!(parent.injection_regions, Duration::from_millis(5));
    }
}
