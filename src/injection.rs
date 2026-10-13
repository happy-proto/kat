use crate::analysis::highlight_named_language_render_data_with_depth;
use crate::display_geometry::ByteOffset;
use crate::display_geometry::DisplayColumn;
use crate::display_geometry::display_column_for_byte_offset;
use crate::document_kind::DocumentKind;
use crate::host_injections::InjectionCandidate;
use crate::host_injections::InjectionProjection;
use crate::host_injections::InjectionVisualAnchor;
use crate::host_injections::InjectionVisualKind;
use crate::host_injections::collect_injection_candidates;
use crate::host_injections::default_visual_level_bump;
use crate::language_runtime::runtime;
use crate::projection::build_candidate_virtual_source;
use crate::projection::map_virtual_nested_regions_to_source;
use crate::projection::map_virtual_regions_to_source;
use crate::projection::map_virtual_spans_to_source;
use crate::projection::shared_leading_indent;
use crate::python_docstrings::render_python_docstring;
use crate::render_data::HighlightRenderData;
use crate::render_data::NestedRegion;
use crate::render_data::RegionSegment;
use crate::render_data::StyledSpan;
use crate::semantic_overlays::github_actions_expression_spans;
#[cfg(test)]
use crate::semantic_overlays::parse_language_tree;
use crate::span_ops::overlay_style_spans;
use crate::theme::Theme;
use crate::timing::RenderTimings;
use crate::visual::collect_visual_regions;
use anyhow::Context;
use anyhow::Result;
use std::ops::Range;
use tree_sitter::Tree;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedInjectionVisual {
    kind: InjectionVisualKind,
    level_bump: usize,
    anchor: InjectionVisualAnchor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CoveredLineRole {
    Content,
    Blank,
    Wrapper,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CoveredLine {
    line_start: usize,
    covered_start: usize,
    covered_end: usize,
    line_end: usize,
    role: CoveredLineRole,
}

#[cfg(test)]
pub(crate) fn collect_top_level_injection_regions(
    document_kind: DocumentKind,
    source: &str,
    theme: &Theme,
    nested_visual_level: usize,
    timings: Option<&mut RenderTimings>,
) -> Result<Vec<NestedRegion>> {
    let tree = parse_language_tree(document_kind.runtime_name(), source)?;
    collect_top_level_injection_regions_from_tree(
        document_kind,
        source,
        &tree,
        theme,
        nested_visual_level,
        timings,
    )
}

pub(crate) fn collect_top_level_injection_regions_from_tree(
    document_kind: DocumentKind,
    source: &str,
    tree: &Tree,
    theme: &Theme,
    nested_visual_level: usize,
    timings: Option<&mut RenderTimings>,
) -> Result<Vec<NestedRegion>> {
    let runtime_name = document_kind.runtime_name();
    let language_runtime = runtime(runtime_name)
        .with_context(|| format!("missing language runtime for {runtime_name}"))?;
    let candidates = collect_injection_candidates(document_kind, language_runtime, tree, source)?;
    let candidates = if language_runtime.injections_query.is_some() {
        merge_adjacent_combined_candidates(source, candidates)
    } else {
        candidates
    };
    let candidates = prune_to_top_level_injection_regions(candidates);
    render_injection_candidates(source, theme, nested_visual_level, timings, candidates)
}

pub(crate) fn render_injection_candidates(
    source: &str,
    theme: &Theme,
    parent_visual_level: usize,
    mut timings: Option<&mut RenderTimings>,
    mut top_level: Vec<InjectionCandidate>,
) -> Result<Vec<NestedRegion>> {
    let mut rendered = Vec::with_capacity(top_level.len());

    for candidate in top_level.drain(..) {
        let visual = resolve_injection_visual(source, &candidate);
        let visual_level = parent_visual_level.saturating_add(visual.level_bump);
        let (virtual_source, source_map) = build_candidate_virtual_source(source, &candidate);
        let child_render = render_virtual_injection_render_data(
            candidate.document_kind,
            &virtual_source,
            candidate.highlight_github_expressions,
            theme,
            visual_level,
            timings.as_deref_mut(),
        )?;
        let layout_segments =
            build_region_segments(source, &candidate.ranges, visual.anchor, visual.kind);
        rendered.push(NestedRegion {
            visual_level,
            resolved_document_kind: child_render.resolved_document_kind,
            visual_kind: visual.kind,
            layout_segments,
            child_regions: map_virtual_regions_to_source(
                source,
                &collect_visual_regions(&child_render.nested_regions),
                &source_map,
            ),
            child_nested_regions: map_virtual_nested_regions_to_source(
                source,
                &child_render.nested_regions,
                &source_map,
            ),
            overlays: map_virtual_spans_to_source(&child_render.spans, &source_map),
            merge_parent_styles: candidate.merge_parent_styles,
        });
    }
    Ok(rendered)
}

pub(crate) fn resolve_injection_visual(
    source: &str,
    candidate: &InjectionCandidate,
) -> ResolvedInjectionVisual {
    let kind = candidate
        .visual_kind
        .unwrap_or_else(|| infer_injection_visual_kind(source, &candidate.ranges));
    let anchor = candidate
        .visual_anchor
        .unwrap_or_else(|| infer_injection_visual_anchor(source, &candidate.ranges, kind));
    let level_bump = candidate
        .visual_level_bump
        .unwrap_or_else(|| default_visual_level_bump(kind));

    ResolvedInjectionVisual {
        kind,
        level_bump,
        anchor,
    }
}

pub(crate) fn infer_injection_visual_kind(
    source: &str,
    ranges: &[Range<usize>],
) -> InjectionVisualKind {
    if ranges_form_standalone_block(source, ranges) {
        InjectionVisualKind::RectBlock
    } else {
        InjectionVisualKind::Transparent
    }
}

pub(crate) fn infer_injection_visual_anchor(
    source: &str,
    ranges: &[Range<usize>],
    visual_kind: InjectionVisualKind,
) -> InjectionVisualAnchor {
    if visual_kind.is_block() && block_visual_prefers_line_start(source, ranges) {
        InjectionVisualAnchor::LineStart
    } else {
        InjectionVisualAnchor::Content
    }
}

pub(crate) fn ranges_form_standalone_block(source: &str, ranges: &[Range<usize>]) -> bool {
    let covered_lines = classify_covered_lines(source, ranges);
    let content_indices = covered_lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (line.role == CoveredLineRole::Content).then_some(index))
        .collect::<Vec<_>>();
    let (Some(&first_content), Some(&last_content)) =
        (content_indices.first(), content_indices.last())
    else {
        return false;
    };

    if content_indices.len() < 2 {
        return false;
    }

    for line in &covered_lines[first_content..=last_content] {
        if line.role == CoveredLineRole::Wrapper {
            return false;
        }

        let prefix = &source[line.line_start..line.covered_start];
        if !prefix.chars().all(|ch| matches!(ch, ' ' | '\t')) {
            return false;
        }

        if line.role == CoveredLineRole::Content
            && source[line.covered_end..line.line_end]
                .chars()
                .any(|ch| !ch.is_whitespace())
        {
            return false;
        }
    }

    true
}

pub(crate) fn classify_covered_lines(source: &str, ranges: &[Range<usize>]) -> Vec<CoveredLine> {
    covered_line_ranges(source, ranges)
        .map(|(line_start, covered_start, covered_end, line_end)| {
            let prefix = &source[line_start..covered_start];
            let covered = &source[covered_start..covered_end];
            let suffix = &source[covered_end..line_end];
            let role = if covered.chars().any(|ch| !ch.is_whitespace()) {
                CoveredLineRole::Content
            } else if prefix.chars().any(|ch| !ch.is_whitespace())
                || suffix.chars().any(|ch| !ch.is_whitespace())
            {
                CoveredLineRole::Wrapper
            } else {
                CoveredLineRole::Blank
            };

            CoveredLine {
                line_start,
                covered_start,
                covered_end,
                line_end,
                role,
            }
        })
        .collect()
}

pub(crate) fn block_visual_prefers_line_start(source: &str, ranges: &[Range<usize>]) -> bool {
    let mut saw_text = false;

    for (_, covered_start, covered_end, _) in covered_line_ranges(source, ranges) {
        if covered_start >= covered_end {
            continue;
        }

        let line_text = &source[covered_start..covered_end];
        let trimmed = line_text.trim_start_matches([' ', '\t']);
        let Some(first) = trimmed.chars().next() else {
            continue;
        };
        saw_text = true;

        if !matches!(first, '/' | '*' | '#') {
            return false;
        }
    }

    saw_text
}

pub(crate) fn covered_line_ranges<'a>(
    source: &'a str,
    ranges: &'a [Range<usize>],
) -> impl Iterator<Item = (usize, usize, usize, usize)> + 'a {
    ranges.iter().flat_map(|range| {
        let mut segments = Vec::new();
        let mut line_start = line_start_offset(source, range.start);
        let line_end_limit = range.end;

        while line_start < line_end_limit {
            let line_end = source[line_start..]
                .find('\n')
                .map(|offset| line_start + offset)
                .unwrap_or(source.len());
            let covered_start = range.start.max(line_start).min(line_end);
            let covered_end = range.end.min(line_end);
            segments.push((line_start, covered_start, covered_end, line_end));

            if line_end >= line_end_limit || line_end == source.len() {
                break;
            }

            line_start = line_end + 1;
        }

        segments
    })
}

pub(crate) fn render_virtual_injection_render_data(
    document_kind: DocumentKind,
    source: &str,
    highlight_github_expressions: bool,
    theme: &Theme,
    nested_visual_level: usize,
    timings: Option<&mut RenderTimings>,
) -> Result<HighlightRenderData> {
    if document_kind.runtime_name() == "python_docstring" {
        return render_python_docstring(document_kind, source, theme, nested_visual_level);
    }

    let mut render_data = highlight_named_language_render_data_with_depth(
        document_kind,
        None,
        source,
        theme,
        nested_visual_level,
        timings,
    )?;

    if highlight_github_expressions {
        render_data.spans =
            overlay_github_actions_expression_styles(source, render_data.spans, theme);
    }

    Ok(render_data)
}

pub(crate) fn overlay_github_actions_expression_styles(
    source: &str,
    parent_spans: Vec<StyledSpan>,
    theme: &Theme,
) -> Vec<StyledSpan> {
    let overlays = github_actions_expression_spans(source)
        .into_iter()
        .filter_map(|span| {
            let text = &source[span.range.clone()];
            theme
                .token_style_for(span.capture, text)
                .map(|style| StyledSpan {
                    range: span.range,
                    style: Some(style),
                    hyperlink: None,
                })
        })
        .collect();

    overlay_style_spans(parent_spans, overlays)
}

pub(crate) fn prune_to_top_level_injection_regions(
    mut candidates: Vec<InjectionCandidate>,
) -> Vec<InjectionCandidate> {
    candidates.sort_by(|left, right| {
        candidate_extent(&left.ranges)
            .start
            .cmp(&candidate_extent(&right.ranges).start)
            .then(
                candidate_extent(&right.ranges)
                    .end
                    .cmp(&candidate_extent(&left.ranges).end),
            )
    });

    let mut pruned: Vec<InjectionCandidate> = Vec::new();

    for candidate in candidates {
        if let Some(last) = pruned.last()
            && candidates_overlap(&candidate, last)
        {
            continue;
        }

        pruned.push(candidate);
    }

    pruned
}

pub(crate) fn candidates_overlap(left: &InjectionCandidate, right: &InjectionCandidate) -> bool {
    left.ranges.iter().any(|left_range| {
        right.ranges.iter().any(|right_range| {
            left_range.start < right_range.end && right_range.start < left_range.end
        })
    })
}

pub(crate) fn merge_adjacent_combined_candidates(
    source: &str,
    mut candidates: Vec<InjectionCandidate>,
) -> Vec<InjectionCandidate> {
    candidates.sort_by(|left, right| {
        candidate_extent(&left.ranges)
            .start
            .cmp(&candidate_extent(&right.ranges).start)
            .then(
                candidate_extent(&left.ranges)
                    .end
                    .cmp(&candidate_extent(&right.ranges).end),
            )
    });

    let mut merged: Vec<InjectionCandidate> = Vec::new();

    for candidate in candidates {
        if let Some(last) = merged.last_mut()
            && last.is_combined
            && candidate.is_combined
            && last.document_kind == candidate.document_kind
            && last.merge_parent_styles == candidate.merge_parent_styles
            && last.strip_shared_indent == candidate.strip_shared_indent
            && last.decode == candidate.decode
            && last.highlight_github_expressions == candidate.highlight_github_expressions
            && last.visual_kind == candidate.visual_kind
            && last.visual_level_bump == candidate.visual_level_bump
            && last.visual_anchor == candidate.visual_anchor
            && matches!(
                (&last.projection, &candidate.projection),
                (
                    InjectionProjection::RawRanges,
                    InjectionProjection::RawRanges
                )
            )
            && can_merge_combined_candidates(source, last, &candidate)
        {
            last.ranges.extend(candidate.ranges);
            last.ranges = normalize_ranges(std::mem::take(&mut last.ranges));
            continue;
        }

        merged.push(candidate);
    }

    merged
}

pub(crate) fn can_merge_combined_candidates(
    source: &str,
    left: &InjectionCandidate,
    right: &InjectionCandidate,
) -> bool {
    let left_extent = candidate_extent(&left.ranges);
    let right_extent = candidate_extent(&right.ranges);
    if left_extent.end > right_extent.start {
        return false;
    }

    source[left_extent.end..right_extent.start]
        .chars()
        .all(|ch| ch.is_whitespace() || matches!(ch, '/' | '*' | '!'))
}

pub(crate) fn candidate_extent(ranges: &[Range<usize>]) -> Range<usize> {
    let start = ranges.first().map_or(0, |range| range.start);
    let end = ranges.last().map_or(start, |range| range.end);
    start..end
}

pub(crate) fn normalize_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|range| range.start < range.end);
    ranges.sort_by(|left, right| left.start.cmp(&right.start).then(left.end.cmp(&right.end)));
    ranges
}

pub(crate) fn build_region_segments(
    source: &str,
    ranges: &[Range<usize>],
    visual_anchor: InjectionVisualAnchor,
    visual_kind: InjectionVisualKind,
) -> Vec<RegionSegment> {
    let shared_indent = shared_leading_indent(source, ranges);
    let mut line_bounds = match visual_kind {
        InjectionVisualKind::Transparent | InjectionVisualKind::TightBlock => {
            build_region_segment_bounds(source, ranges, shared_indent, visual_anchor)
        }
        InjectionVisualKind::RectBlock => {
            build_block_region_segments(source, ranges, visual_anchor)
        }
    };

    line_bounds.sort_by_key(|line| (line.line_start, line.left, line.text_end));
    line_bounds.dedup();

    line_bounds
}

pub(crate) fn build_block_region_segments(
    source: &str,
    ranges: &[Range<usize>],
    visual_anchor: InjectionVisualAnchor,
) -> Vec<RegionSegment> {
    let covered_lines = classify_covered_lines(source, ranges);
    let Some(block_lines) = content_block_lines(&covered_lines) else {
        return build_region_segment_bounds(
            source,
            ranges,
            shared_leading_indent(source, ranges),
            visual_anchor,
        );
    };
    let block_left_column = block_lines
        .iter()
        .filter(|line| line.role == CoveredLineRole::Content)
        .map(|line| {
            let line_text = &source[line.line_start..line.line_end];
            let left_in_line = ByteOffset::new(content_start_offset(source, *line));
            display_column_for_byte_offset(line_text, left_in_line)
        })
        .min()
        .unwrap_or(DisplayColumn::new(0));

    block_lines
        .iter()
        .filter(|line| line.role != CoveredLineRole::Wrapper)
        .map(|line| {
            let (left, left_column_override) = match visual_anchor {
                InjectionVisualAnchor::Content => (
                    line.line_start + content_start_offset(source, *line),
                    Some(block_left_column),
                ),
                InjectionVisualAnchor::LineStart => (line.line_start, None),
            };

            RegionSegment {
                line_start: line.line_start,
                left,
                text_end: line.line_end,
                left_column_override,
            }
        })
        .collect()
}

pub(crate) fn content_block_lines(covered_lines: &[CoveredLine]) -> Option<&[CoveredLine]> {
    let first_content = covered_lines
        .iter()
        .position(|line| line.role == CoveredLineRole::Content)?;
    let last_content = covered_lines
        .iter()
        .rposition(|line| line.role == CoveredLineRole::Content)?;
    Some(&covered_lines[first_content..=last_content])
}

pub(crate) fn content_start_offset(source: &str, line: CoveredLine) -> usize {
    let indent_bytes = source[line.covered_start..line.covered_end]
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .map(char::len_utf8)
        .sum::<usize>();
    line.covered_start + indent_bytes - line.line_start
}

pub(crate) fn build_region_segment_bounds(
    source: &str,
    ranges: &[Range<usize>],
    shared_indent: usize,
    visual_anchor: InjectionVisualAnchor,
) -> Vec<RegionSegment> {
    covered_line_ranges(source, ranges)
        .map(|(line_start, covered_start, covered_end, line_end)| {
            build_region_segment_for_line(
                source,
                CoveredLine {
                    line_start,
                    covered_start,
                    covered_end,
                    line_end,
                    role: CoveredLineRole::Content,
                },
                shared_indent,
                visual_anchor,
            )
        })
        .collect()
}

pub(crate) fn build_region_segment_for_line(
    source: &str,
    line: CoveredLine,
    shared_indent: usize,
    visual_anchor: InjectionVisualAnchor,
) -> RegionSegment {
    let indent_bytes = source[line.covered_start..line.line_end]
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .map(char::len_utf8)
        .take(shared_indent)
        .sum::<usize>();
    let left = match visual_anchor {
        InjectionVisualAnchor::Content => (line.covered_start + indent_bytes).min(line.line_end),
        InjectionVisualAnchor::LineStart => line.line_start,
    };

    RegionSegment {
        line_start: line.line_start,
        left,
        text_end: line.line_end,
        left_column_override: None,
    }
}

pub(crate) fn line_start_offset(source: &str, offset: usize) -> usize {
    source[..offset]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0)
}
