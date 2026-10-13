use crate::render_data::NestedRegion;
use crate::render_data::StyledSpan;
use crate::theme::TokenStyle;
use std::ops::Range;

pub(crate) fn push_span(
    spans: &mut Vec<StyledSpan>,
    range: Range<usize>,
    style: Option<TokenStyle>,
    hyperlink: Option<String>,
) {
    if range.start >= range.end {
        return;
    }

    if let Some(last) = spans.last_mut()
        && last.range.end == range.start
        && last.style == style
        && last.hyperlink == hyperlink
    {
        last.range.end = range.end;
        return;
    }

    spans.push(StyledSpan {
        range,
        style,
        hyperlink,
    });
}

pub(crate) fn overlay_nested_region(
    parent_spans: Vec<StyledSpan>,
    region: &NestedRegion,
) -> Vec<StyledSpan> {
    let spans = apply_overlay_spans(parent_spans, &region.overlays, region.merge_parent_styles);
    region
        .child_nested_regions
        .iter()
        .fold(spans, overlay_nested_region)
}

pub(crate) fn overlay_style_spans(
    parent_spans: Vec<StyledSpan>,
    overlay_spans: Vec<StyledSpan>,
) -> Vec<StyledSpan> {
    apply_overlay_spans(parent_spans, &overlay_spans, false)
}

pub(crate) fn style_covering_span(
    spans: &[StyledSpan],
    start: usize,
    end: usize,
) -> Option<TokenStyle> {
    spans
        .iter()
        .find(|span| span.range.start <= start && span.range.end >= end)
        .and_then(|span| span.style)
}

pub(crate) fn apply_overlay_spans(
    parent_spans: Vec<StyledSpan>,
    overlay_spans: &[StyledSpan],
    merge_parent_styles: bool,
) -> Vec<StyledSpan> {
    if overlay_spans.is_empty() {
        return parent_spans;
    }

    let mut result = Vec::with_capacity(parent_spans.len().saturating_add(overlay_spans.len()));
    let mut overlay_index = 0;

    for parent in parent_spans {
        let parent_start = parent.range.start;
        let parent_end = parent.range.end;
        let mut cursor = parent_start;

        while overlay_index < overlay_spans.len()
            && overlay_spans[overlay_index].range.end <= cursor
        {
            overlay_index += 1;
        }

        let mut active_overlay_index = overlay_index;
        while active_overlay_index < overlay_spans.len()
            && overlay_spans[active_overlay_index].range.start < parent_end
        {
            let overlay = &overlay_spans[active_overlay_index];
            if cursor < overlay.range.start {
                push_span(
                    &mut result,
                    cursor..overlay.range.start.min(parent_end),
                    parent.style,
                    parent.hyperlink.clone(),
                );
                cursor = overlay.range.start.min(parent_end);
            }

            let start = cursor.max(overlay.range.start);
            let end = parent_end.min(overlay.range.end);
            if start < end {
                push_span(
                    &mut result,
                    start..end,
                    resolve_overlay_style(parent.style, overlay.style, merge_parent_styles),
                    resolve_overlay_hyperlink(
                        parent.hyperlink.as_deref(),
                        overlay.hyperlink.as_deref(),
                    ),
                );
                cursor = end;
            }

            if overlay.range.end <= parent_end {
                active_overlay_index += 1;
                overlay_index = active_overlay_index;
            } else {
                break;
            }
        }

        if cursor < parent_end {
            push_span(
                &mut result,
                cursor..parent_end,
                parent.style,
                parent.hyperlink.clone(),
            );
        }
    }

    result
}

pub(crate) fn resolve_overlay_hyperlink(
    parent: Option<&str>,
    overlay: Option<&str>,
) -> Option<String> {
    overlay.or(parent).map(str::to_owned)
}

pub(crate) fn resolve_overlay_style(
    parent_style: Option<TokenStyle>,
    overlay_style: Option<TokenStyle>,
    merge_parent_styles: bool,
) -> Option<TokenStyle> {
    match (merge_parent_styles, parent_style, overlay_style) {
        (true, Some(parent), Some(overlay)) => Some(parent.merge(overlay)),
        (true, Some(parent), None) => Some(parent),
        (true, None, Some(overlay)) => Some(overlay),
        (true, None, None) => None,
        (false, _, Some(overlay)) => Some(overlay),
        (false, parent, None) => parent,
    }
}

pub(crate) fn span_covering_range_from<'a>(
    spans: &'a [StyledSpan],
    cursor: &mut usize,
    start: usize,
    end: usize,
) -> Option<&'a StyledSpan> {
    while *cursor < spans.len() && spans[*cursor].range.end <= start {
        *cursor += 1;
    }

    spans
        .get(*cursor)
        .filter(|span| span.range.start <= start && span.range.end >= end)
}
