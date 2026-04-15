use std::collections::HashMap;

use serde::Serialize;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    StyledSpan, VisualRegion,
    display_geometry::display_width,
    style_covering_span_from,
    theme::{ColorMode, Theme, TokenStyle, TokenStyleSnapshot},
};

#[derive(Clone, Debug)]
pub(crate) struct LayoutDocument {
    color_mode: ColorMode,
    rows: Vec<LayoutRow>,
}

impl LayoutDocument {
    pub(crate) fn from_visual(
        source: &str,
        spans: &[StyledSpan],
        regions: &[VisualRegion],
        theme: Theme,
        terminal_width: Option<usize>,
    ) -> Self {
        let mut rows = build_rows(source, spans, terminal_width);
        let line_rows = rows_by_line_start(&rows);
        paint_backgrounds(source, regions, theme, &line_rows, &mut rows);

        Self {
            color_mode: theme.color_mode(),
            rows,
        }
    }

    pub(crate) fn rows(&self) -> &[LayoutRow] {
        &self.rows
    }

    pub(crate) fn snapshot(&self) -> LayoutSnapshot {
        LayoutSnapshot {
            color_mode: self.color_mode,
            rows: self
                .rows
                .iter()
                .map(|row| row.snapshot(self.color_mode))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LayoutRow {
    pub(crate) source_line_start: usize,
    pub(crate) wrapped_from_column: usize,
    pub(crate) wrapped_until_column: usize,
    pub(crate) display_width: usize,
    pub(crate) cells: Vec<LayoutCell>,
    pub(crate) background_runs: Vec<LayoutBackgroundRun>,
}

impl LayoutRow {
    fn snapshot(&self, color_mode: ColorMode) -> LayoutRowSnapshot {
        LayoutRowSnapshot {
            source_line_start: self.source_line_start,
            wrapped_from_column: self.wrapped_from_column,
            wrapped_until_column: self.wrapped_until_column,
            display_width: self.display_width,
            text: self.cells.iter().map(|cell| cell.text.as_str()).collect(),
            cells: self
                .cells
                .iter()
                .map(|cell| cell.snapshot(color_mode))
                .collect(),
            background_runs: self
                .background_runs
                .iter()
                .map(|run| run.snapshot(color_mode))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LayoutCell {
    pub(crate) column: usize,
    pub(crate) width: usize,
    pub(crate) text: String,
    pub(crate) style: Option<TokenStyle>,
}

impl LayoutCell {
    fn snapshot(&self, color_mode: ColorMode) -> LayoutCellSnapshot {
        LayoutCellSnapshot {
            column: self.column,
            width: self.width,
            text: self.text.clone(),
            style: self.style.map(|style| style.snapshot(color_mode)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LayoutBackgroundRun {
    pub(crate) start_column: usize,
    pub(crate) end_column: usize,
    pub(crate) visual_level: usize,
    pub(crate) style: TokenStyle,
}

impl LayoutBackgroundRun {
    fn snapshot(&self, color_mode: ColorMode) -> LayoutBackgroundRunSnapshot {
        LayoutBackgroundRunSnapshot {
            start_column: self.start_column,
            end_column: self.end_column,
            visual_level: self.visual_level,
            style: self.style.snapshot(color_mode),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LayoutSnapshot {
    pub color_mode: ColorMode,
    pub rows: Vec<LayoutRowSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LayoutRowSnapshot {
    pub source_line_start: usize,
    pub wrapped_from_column: usize,
    pub wrapped_until_column: usize,
    pub display_width: usize,
    pub text: String,
    pub cells: Vec<LayoutCellSnapshot>,
    pub background_runs: Vec<LayoutBackgroundRunSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LayoutCellSnapshot {
    pub column: usize,
    pub width: usize,
    pub text: String,
    pub style: Option<TokenStyleSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LayoutBackgroundRunSnapshot {
    pub start_column: usize,
    pub end_column: usize,
    pub visual_level: usize,
    pub style: TokenStyleSnapshot,
}

#[derive(Clone, Copy, Debug)]
struct RowRef {
    row_index: usize,
    start_column: usize,
    end_column: usize,
}

#[derive(Clone, Copy, Debug)]
struct BackgroundSlice {
    row_index: usize,
    start_column: usize,
    end_column: usize,
}

#[derive(Clone, Copy, Debug)]
struct GraphemeSpan {
    byte_start: usize,
    byte_end: usize,
    column_start: usize,
    column_end: usize,
    width: usize,
}

fn build_rows(source: &str, spans: &[StyledSpan], terminal_width: Option<usize>) -> Vec<LayoutRow> {
    let mut rows = Vec::new();
    let mut span_cursor = 0usize;
    let wrap_width = terminal_width.filter(|width| *width > 0);

    let mut line_start = 0usize;
    while line_start <= source.len() {
        let line_end = source[line_start..]
            .find('\n')
            .map(|offset| line_start + offset)
            .unwrap_or(source.len());
        let line_text = &source[line_start..line_end];
        let graphemes = grapheme_spans(line_text, line_start);

        if graphemes.is_empty() {
            rows.push(LayoutRow {
                source_line_start: line_start,
                wrapped_from_column: 0,
                wrapped_until_column: 0,
                display_width: 0,
                cells: Vec::new(),
                background_runs: Vec::new(),
            });
        } else {
            let wrapped_rows = wrap_graphemes(&graphemes, wrap_width);
            for row in wrapped_rows {
                let mut cells = Vec::new();
                for grapheme in &graphemes[row.grapheme_start..row.grapheme_end] {
                    let style = style_covering_span_from(
                        spans,
                        &mut span_cursor,
                        grapheme.byte_start,
                        grapheme.byte_end,
                    );
                    cells.push(LayoutCell {
                        column: grapheme.column_start - row.start_column,
                        width: grapheme.width,
                        text: source[grapheme.byte_start..grapheme.byte_end].to_owned(),
                        style,
                    });
                }
                rows.push(LayoutRow {
                    source_line_start: line_start,
                    wrapped_from_column: row.start_column,
                    wrapped_until_column: row.end_column,
                    display_width: row.end_column.saturating_sub(row.start_column),
                    cells,
                    background_runs: Vec::new(),
                });
            }
        }

        if line_end == source.len() {
            break;
        }
        line_start = line_end + 1;
    }

    rows
}

fn rows_by_line_start(rows: &[LayoutRow]) -> HashMap<usize, Vec<RowRef>> {
    let mut line_rows = HashMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        line_rows
            .entry(row.source_line_start)
            .or_insert_with(Vec::new)
            .push(RowRef {
                row_index,
                start_column: row.wrapped_from_column,
                end_column: row.wrapped_until_column,
            });
    }
    line_rows
}

fn paint_backgrounds(
    source: &str,
    regions: &[VisualRegion],
    theme: Theme,
    line_rows: &HashMap<usize, Vec<RowRef>>,
    rows: &mut [LayoutRow],
) {
    let mut pending_runs = vec![Vec::<LayoutBackgroundRun>::new(); rows.len()];

    for region in regions {
        let Some(style) = theme.nested_region_tint(region.visual_level) else {
            continue;
        };

        match region.visual_kind {
            crate::host_injections::InjectionVisualKind::Transparent => {}
            crate::host_injections::InjectionVisualKind::TightBlock => {
                for segment in &region.segments {
                    for slice in segment_content_slices(source, segment, line_rows) {
                        pending_runs[slice.row_index].push(LayoutBackgroundRun {
                            start_column: slice.start_column,
                            end_column: slice.end_column,
                            visual_level: region.visual_level,
                            style,
                        });
                    }
                }
            }
            crate::host_injections::InjectionVisualKind::RectBlock => {
                for segment in &region.segments {
                    for slice in segment_rect_slices(source, segment, line_rows) {
                        pending_runs[slice.row_index].push(LayoutBackgroundRun {
                            start_column: slice.start_column,
                            end_column: slice.end_column,
                            visual_level: region.visual_level,
                            style,
                        });
                    }
                }
            }
            crate::host_injections::InjectionVisualKind::ScopeBlock => {
                let mut slices = Vec::new();
                let mut right_edge = 0usize;

                for segment in &region.segments {
                    for slice in segment_content_slices(source, segment, line_rows) {
                        right_edge = right_edge.max(slice.end_column);
                        slices.push(slice);
                    }
                }

                let mut row_lefts = HashMap::<usize, usize>::new();
                for slice in slices {
                    row_lefts
                        .entry(slice.row_index)
                        .and_modify(|left| *left = (*left).min(slice.start_column))
                        .or_insert(slice.start_column);
                }

                for (row_index, left_edge) in row_lefts {
                    pending_runs[row_index].push(LayoutBackgroundRun {
                        start_column: left_edge,
                        end_column: right_edge,
                        visual_level: region.visual_level,
                        style,
                    });
                }
            }
        }
    }

    for (row, raw_runs) in rows.iter_mut().zip(pending_runs) {
        if raw_runs.is_empty() {
            continue;
        }

        let max_end = raw_runs
            .iter()
            .map(|run| run.end_column)
            .max()
            .unwrap_or(row.display_width);
        row.display_width = row.display_width.max(max_end);
        row.background_runs = resolve_background_runs(raw_runs, row.display_width);
    }
}

fn resolve_background_runs(
    runs: Vec<LayoutBackgroundRun>,
    row_width: usize,
) -> Vec<LayoutBackgroundRun> {
    if row_width == 0 {
        return Vec::new();
    }

    let mut cells = vec![None::<LayoutBackgroundRun>; row_width];
    for run in runs {
        let end = run.end_column.min(row_width);
        for cell in cells
            .iter_mut()
            .take(end)
            .skip(run.start_column.min(row_width))
        {
            if cell
                .as_ref()
                .is_none_or(|current| run.visual_level >= current.visual_level)
            {
                *cell = Some(run);
            }
        }
    }

    let mut resolved = Vec::new();
    let mut current: Option<LayoutBackgroundRun> = None;

    for (column, cell) in cells.into_iter().enumerate() {
        match (current.take(), cell) {
            (Some(mut active), Some(next))
                if active.visual_level == next.visual_level && active.style == next.style =>
            {
                active.end_column = column + 1;
                current = Some(active);
            }
            (Some(active), Some(next)) => {
                resolved.push(active);
                current = Some(LayoutBackgroundRun {
                    start_column: column,
                    end_column: column + 1,
                    ..next
                });
            }
            (Some(active), None) => resolved.push(active),
            (None, Some(next)) => {
                current = Some(LayoutBackgroundRun {
                    start_column: column,
                    end_column: column + 1,
                    ..next
                });
            }
            (None, None) => {}
        }
    }

    if let Some(active) = current {
        resolved.push(active);
    }

    resolved
}

fn segment_content_slices(
    source: &str,
    segment: &crate::RegionSegment,
    line_rows: &HashMap<usize, Vec<RowRef>>,
) -> Vec<BackgroundSlice> {
    let Some(rows) = line_rows.get(&segment.line_start) else {
        return Vec::new();
    };

    let left_column = display_column(source, segment.line_start, segment.left);
    let content_end = trim_trailing_whitespace(source, segment.left, segment.text_end);
    let right_column = display_column(source, segment.line_start, content_end);
    if right_column <= left_column {
        return Vec::new();
    }

    rows.iter()
        .filter_map(|row| intersect_slice(*row, left_column, right_column))
        .collect()
}

fn segment_rect_slices(
    source: &str,
    segment: &crate::RegionSegment,
    line_rows: &HashMap<usize, Vec<RowRef>>,
) -> Vec<BackgroundSlice> {
    let Some(rows) = line_rows.get(&segment.line_start) else {
        return Vec::new();
    };

    let left_column = display_column(source, segment.line_start, segment.left);
    let right_column = display_column(source, segment.line_start, segment.text_end)
        + segment.right_padding.as_usize();
    if right_column <= left_column {
        return Vec::new();
    }
    let line_text_right = rows.iter().map(|row| row.end_column).max().unwrap_or(0);

    rows.iter()
        .filter_map(|row| {
            let row_right = if row.end_column == line_text_right {
                right_column
            } else {
                right_column.min(row.end_column)
            };
            intersect_rect_slice(*row, left_column, row_right)
        })
        .collect()
}

fn intersect_slice(
    row: RowRef,
    segment_left: usize,
    segment_right: usize,
) -> Option<BackgroundSlice> {
    let start = segment_left.max(row.start_column);
    let end = segment_right.min(row.end_column);
    (start < end).then_some(BackgroundSlice {
        row_index: row.row_index,
        start_column: start - row.start_column,
        end_column: end - row.start_column,
    })
}

fn intersect_rect_slice(
    row: RowRef,
    segment_left: usize,
    segment_right: usize,
) -> Option<BackgroundSlice> {
    let start = segment_left.max(row.start_column);
    let end = segment_right.max(row.start_column);
    (start < end).then_some(BackgroundSlice {
        row_index: row.row_index,
        start_column: start - row.start_column,
        end_column: end - row.start_column,
    })
}

fn display_column(source: &str, line_start: usize, byte_offset: usize) -> usize {
    display_width(&source[line_start..byte_offset]).as_usize()
}

fn trim_trailing_whitespace(source: &str, start: usize, end: usize) -> usize {
    let mut trimmed = end;
    while trimmed > start {
        let ch = source[..trimmed]
            .chars()
            .next_back()
            .expect("expected utf-8 char while trimming trailing whitespace");
        if ch.is_whitespace() {
            trimmed -= ch.len_utf8();
        } else {
            break;
        }
    }
    trimmed
}

fn grapheme_spans(line_text: &str, line_start: usize) -> Vec<GraphemeSpan> {
    let mut spans = Vec::new();
    let mut column = 0usize;
    for (byte_offset, grapheme) in UnicodeSegmentation::grapheme_indices(line_text, true) {
        let width = UnicodeWidthStr::width(grapheme);
        spans.push(GraphemeSpan {
            byte_start: line_start + byte_offset,
            byte_end: line_start + byte_offset + grapheme.len(),
            column_start: column,
            column_end: column + width,
            width,
        });
        column += width;
    }
    spans
}

#[derive(Clone, Copy, Debug)]
struct WrappedRowRange {
    grapheme_start: usize,
    grapheme_end: usize,
    start_column: usize,
    end_column: usize,
}

fn wrap_graphemes(
    graphemes: &[GraphemeSpan],
    terminal_width: Option<usize>,
) -> Vec<WrappedRowRange> {
    let Some(width) = terminal_width else {
        return vec![WrappedRowRange {
            grapheme_start: 0,
            grapheme_end: graphemes.len(),
            start_column: graphemes
                .first()
                .map_or(0, |grapheme| grapheme.column_start),
            end_column: graphemes.last().map_or(0, |grapheme| grapheme.column_end),
        }];
    };

    let mut rows = Vec::new();
    let mut row_start = 0usize;

    while row_start < graphemes.len() {
        let start_column = graphemes[row_start].column_start;
        let mut row_end = row_start;
        let mut row_width = 0usize;

        while row_end < graphemes.len() {
            let width_if_added = row_width + graphemes[row_end].width;
            if row_width > 0 && width_if_added > width {
                break;
            }
            row_width = width_if_added;
            row_end += 1;
            if row_width == width {
                break;
            }
        }

        rows.push(WrappedRowRange {
            grapheme_start: row_start,
            grapheme_end: row_end,
            start_column,
            end_column: graphemes[row_end - 1].column_end,
        });
        row_start = row_end;
    }

    rows
}
