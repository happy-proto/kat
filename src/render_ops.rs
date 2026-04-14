use std::ops::Range;

use serde::Serialize;

use crate::{
    FlatRegionSegment, StyledSpan, flatten_region_segments, style_covering_span_from,
    terminal::escape_control_sequences,
    theme::{ColorMode, Theme, TokenStyle, TokenStyleSnapshot},
    visual::VisualDocument,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderPlan {
    ops: Vec<RenderOp>,
}

impl RenderPlan {
    pub(crate) fn compile(source: &str, visual: &VisualDocument, theme: Theme) -> Self {
        let mut builder = RenderPlanBuilder::new(theme);
        let mut line_start = 0;
        let flat_segments = flatten_region_segments(visual.regions());
        let mut span_start_index = 0;
        let mut span_end_index = 0;
        let mut segment_index = 0;

        while line_start <= source.len() {
            let line_end = source[line_start..]
                .find('\n')
                .map(|offset| line_start + offset)
                .unwrap_or(source.len());

            while span_start_index < visual.spans().len()
                && visual.spans()[span_start_index].range.end <= line_start
            {
                span_start_index += 1;
            }
            span_end_index = span_end_index.max(span_start_index);
            while span_end_index < visual.spans().len()
                && visual.spans()[span_end_index].range.start < line_end
            {
                span_end_index += 1;
            }

            while segment_index < flat_segments.len()
                && (flat_segments[segment_index].line_start < line_start
                    || (flat_segments[segment_index].line_start == line_start
                        && flat_segments[segment_index].right <= line_start))
            {
                segment_index += 1;
            }
            let line_segment_start = segment_index;
            while segment_index < flat_segments.len()
                && flat_segments[segment_index].line_start == line_start
            {
                segment_index += 1;
            }

            compile_line_text(
                &mut builder,
                source,
                line_start..line_end,
                &visual.spans()[span_start_index..span_end_index],
                &flat_segments[line_segment_start..segment_index],
            );
            compile_line_padding(
                &mut builder,
                line_end,
                &flat_segments[line_segment_start..segment_index],
            );
            builder.reset_style();

            if line_end == source.len() {
                break;
            }

            builder.push_newline();
            line_start = line_end + 1;
        }

        Self { ops: builder.ops }
    }

    pub(crate) fn encode(&self, theme: Theme) -> String {
        let mut rendered = String::new();
        let mut active_style = None;

        for op in &self.ops {
            match op {
                RenderOp::SetStyle { style } => {
                    let transition = style.render_transition_from(active_style, theme.color_mode());
                    if !transition.is_empty() {
                        rendered.push_str(&transition);
                    }
                    active_style = Some(*style);
                }
                RenderOp::ResetStyle => {
                    if active_style.take().is_some() {
                        rendered.push_str("\x1b[0m");
                    }
                }
                RenderOp::Text { text } => rendered.push_str(text),
                RenderOp::Newline => rendered.push('\n'),
            }
        }

        rendered
    }

    pub(crate) fn snapshot(&self, theme: Theme) -> RenderPlanSnapshot {
        RenderPlanSnapshot {
            color_mode: theme.color_mode(),
            ops: self
                .ops
                .iter()
                .map(|op| op.snapshot(theme.color_mode()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RenderOp {
    SetStyle { style: TokenStyle },
    ResetStyle,
    Text { text: String },
    Newline,
}

impl RenderOp {
    fn snapshot(&self, color_mode: ColorMode) -> RenderOpSnapshot {
        match self {
            Self::SetStyle { style } => RenderOpSnapshot::SetStyle {
                style: style.snapshot(color_mode),
            },
            Self::ResetStyle => RenderOpSnapshot::ResetStyle,
            Self::Text { text } => RenderOpSnapshot::Text {
                text: escape_control_sequences(text),
            },
            Self::Newline => RenderOpSnapshot::Newline,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RenderPlanSnapshot {
    pub color_mode: ColorMode,
    pub ops: Vec<RenderOpSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(crate) enum RenderOpSnapshot {
    SetStyle { style: TokenStyleSnapshot },
    ResetStyle,
    Text { text: String },
    Newline,
}

struct RenderPlanBuilder {
    theme: Theme,
    active_style: Option<TokenStyle>,
    ops: Vec<RenderOp>,
}

impl RenderPlanBuilder {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            active_style: None,
            ops: Vec::new(),
        }
    }

    fn push_text(&mut self, text: &str, style: Option<TokenStyle>) {
        let style = style.filter(|style| !style.renders_as_plain_text(self.theme.color_mode()));
        match style {
            Some(style) => {
                if self.active_style != Some(style) {
                    self.ops.push(RenderOp::SetStyle { style });
                    self.active_style = Some(style);
                }
            }
            None => self.reset_style(),
        }

        if !text.is_empty() {
            self.ops.push(RenderOp::Text {
                text: text.to_owned(),
            });
        }
    }

    fn reset_style(&mut self) {
        if self.active_style.take().is_some() {
            self.ops.push(RenderOp::ResetStyle);
        }
    }

    fn push_newline(&mut self) {
        self.ops.push(RenderOp::Newline);
    }
}

fn compile_line_text(
    builder: &mut RenderPlanBuilder,
    source: &str,
    line_range: Range<usize>,
    spans: &[StyledSpan],
    line_segments: &[FlatRegionSegment],
) {
    let line_start = line_range.start;
    let line_end = line_range.end;
    if line_start == line_end {
        return;
    }

    let mut boundaries = vec![line_start, line_end];

    for span in spans {
        if span.range.start < line_end && span.range.end > line_start {
            boundaries.push(span.range.start.max(line_start));
            boundaries.push(span.range.end.min(line_end));
        }
    }

    for segment in line_segments {
        if segment.left < line_end && segment.text_end > line_start {
            boundaries.push(segment.left.max(line_start));
            boundaries.push(segment.text_end.min(line_end));
        }
    }

    boundaries.sort_unstable();
    boundaries.dedup();
    let mut span_cursor = 0;

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start == end {
            continue;
        }

        let text = &source[start..end];
        let style = merged_visual_style(
            spans,
            line_segments,
            &mut span_cursor,
            start,
            end,
            builder.theme,
        );
        builder.push_text(text, style);
    }
}

fn compile_line_padding(
    builder: &mut RenderPlanBuilder,
    line_end: usize,
    line_segments: &[FlatRegionSegment],
) {
    let mut boundaries = vec![line_end];
    for segment in line_segments {
        if segment.right > line_end {
            boundaries.push(segment.right);
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    if boundaries.len() <= 1 {
        return;
    }

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start == end {
            continue;
        }

        let style = top_region_style(line_segments, start, end, builder.theme);
        if let Some(style) = style {
            builder.push_text(&" ".repeat(end - start), Some(style));
        }
    }
}

fn merged_visual_style(
    spans: &[StyledSpan],
    line_segments: &[FlatRegionSegment],
    span_cursor: &mut usize,
    start: usize,
    end: usize,
    theme: Theme,
) -> Option<TokenStyle> {
    let foreground = style_covering_span_from(spans, span_cursor, start, end);
    let background = top_region_style(line_segments, start, end, theme);

    match (foreground, background) {
        (Some(foreground), Some(background)) => Some(foreground.with_background_under(background)),
        (Some(foreground), None) => Some(foreground),
        (None, Some(background)) => Some(background),
        (None, None) => None,
    }
}

fn top_region_style(
    line_segments: &[FlatRegionSegment],
    start: usize,
    end: usize,
    theme: Theme,
) -> Option<TokenStyle> {
    line_segments
        .iter()
        .filter(|segment| segment.left <= start && segment.right >= end)
        .max_by_key(|segment| segment.visual_level)
        .and_then(|segment| region_style(*segment, theme))
}

fn region_style(segment: FlatRegionSegment, theme: Theme) -> Option<TokenStyle> {
    match segment.visual_kind {
        crate::host_injections::InjectionVisualKind::Transparent => None,
        crate::host_injections::InjectionVisualKind::TightBlock
        | crate::host_injections::InjectionVisualKind::RectBlock
        | crate::host_injections::InjectionVisualKind::ScopeBlock => {
            theme.nested_region_tint(segment.visual_level)
        }
    }
}
