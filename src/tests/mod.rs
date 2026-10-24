mod corpus;
mod detection;
mod highlighting;
mod injections;
mod layout;
mod nushell;
mod pipeline;
mod semantics;

use crate::language_runtime::RuntimeId;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    analysis::{AnalysisSnapshot, NestedRegionSnapshot, RegionSegmentSnapshot},
    display_geometry::{display_width, strip_ansi},
    document_kind::DocumentKind,
    document_kind::{DocumentProfile, yaml_document_kind},
    language_runtime::runtime,
    layout::{LayoutRowSnapshot, LayoutSnapshot},
    render_ops::{RenderOpSnapshot, RenderPlanSnapshot},
    sql_dialect::detect_sql_dialect,
    theme::{ColorMode, Theme},
};
use crate::{
    api::analyze_with_theme, api::debug_analysis_json, api::debug_layout_json,
    api::debug_render_ops_json, api::debug_semantics, api::debug_terminal_json,
    api::debug_visual_json, api::highlight_named_language, api::render_plan_with_theme,
    detection::detect_document_kind, document::render_with_theme,
    document_kind::plain_document_kind, host_injections::InjectionVisualKind,
    injection::collect_top_level_injection_regions, render_data::NestedRegion,
    render_data::RegionSegment, render_data::StyledSpan, semantic_overlays::semantic_capture_spans,
    span_ops::overlay_nested_region, span_ops::overlay_style_spans,
};
use anstyle::RgbColor;
use serde_json::Value;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

fn analysis_snapshot_for_path(path: &Path, source: &str, theme: &Theme) -> AnalysisSnapshot {
    analyze_with_theme(Some(path), source, theme)
        .unwrap_or_else(|error| panic!("failed to analyze {}: {error}", path.display()))
        .snapshot()
}

fn render_plan_snapshot_for_path(path: &Path, source: &str, theme: &Theme) -> RenderPlanSnapshot {
    render_plan_with_theme(Some(path), source, theme)
        .unwrap_or_else(|error| panic!("failed to build render plan {}: {error}", path.display()))
        .snapshot(*theme)
}

fn layout_snapshot_for_path(
    path: &Path,
    source: &str,
    theme: &Theme,
    terminal_width: usize,
) -> LayoutSnapshot {
    crate::api::layout_with_theme(Some(path), source, theme, Some(terminal_width))
        .unwrap_or_else(|error| panic!("failed to build layout {}: {error}", path.display()))
        .snapshot()
}

fn nested_regions(regions: &[NestedRegionSnapshot]) -> Vec<&NestedRegionSnapshot> {
    let mut flattened = Vec::new();
    for region in regions {
        flattened.push(region);
        flattened.extend(nested_regions(&region.child_nested_regions));
    }
    flattened
}

fn nested_runtime_names(snapshot: &AnalysisSnapshot) -> Vec<&'static str> {
    nested_regions(&snapshot.nested_regions)
        .into_iter()
        .map(|region| region.resolved_document_kind.runtime_name)
        .collect()
}

fn find_nested_region<'a>(
    snapshot: &'a AnalysisSnapshot,
    runtime_name: &str,
    line_fragment: &str,
    source: &str,
) -> &'a NestedRegionSnapshot {
    let line_start = line_start_containing(source, line_fragment);
    nested_regions(&snapshot.nested_regions)
            .into_iter()
            .find(|region| {
                region.resolved_document_kind.runtime_name == runtime_name
                    && region
                        .layout_segments
                        .iter()
                        .any(|segment| segment.line_start == line_start)
            })
            .unwrap_or_else(|| {
                panic!(
                    "expected nested region runtime {runtime_name:?} to cover line fragment {line_fragment:?}"
                )
            })
}

fn region_covers_line(region: &NestedRegionSnapshot, source: &str, line_fragment: &str) -> bool {
    let line_start = line_start_containing(source, line_fragment);
    region
        .layout_segments
        .iter()
        .any(|segment| segment.line_start == line_start)
}

fn line_start_containing(source: &str, needle: &str) -> usize {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected source to contain line fragment {needle:?}"));
    crate::injection::line_start_offset(source, start)
}

fn line_start_for_line_number(source: &str, line_number: usize) -> usize {
    if line_number == 0 {
        return 0;
    }

    source
        .match_indices('\n')
        .nth(line_number - 1)
        .map(|(index, _)| index + 1)
        .unwrap_or_else(|| panic!("expected source to have line number {line_number}"))
}

fn line_text_at(source: &str, line_start: usize) -> &str {
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    &source[line_start..line_end]
}

fn segment_for_line<'a>(
    region: &'a NestedRegionSnapshot,
    source: &str,
    line_fragment: &str,
) -> &'a RegionSegmentSnapshot {
    let line_start = line_start_containing(source, line_fragment);
    region
        .layout_segments
        .iter()
        .find(|segment| segment.line_start == line_start)
        .unwrap_or_else(|| panic!("expected region to cover line fragment {line_fragment:?}"))
}

fn segment_left_column(source: &str, segment: &RegionSegmentSnapshot) -> usize {
    segment
        .left_column_override
        .unwrap_or_else(|| display_width(&source[segment.line_start..segment.left]).as_usize())
}

fn render_plan_text(snapshot: &RenderPlanSnapshot) -> String {
    let mut text = String::new();
    for op in &snapshot.ops {
        match op {
            RenderOpSnapshot::Text { text: chunk } => text.push_str(chunk),
            RenderOpSnapshot::Newline => text.push('\n'),
            RenderOpSnapshot::StartHyperlink { .. }
            | RenderOpSnapshot::EndHyperlink
            | RenderOpSnapshot::SetStyle { .. }
            | RenderOpSnapshot::ResetStyle => {}
        }
    }
    text
}

fn render_plan_lines(snapshot: &RenderPlanSnapshot) -> Vec<Vec<RenderOpSnapshot>> {
    let mut lines = vec![Vec::new()];
    for op in &snapshot.ops {
        match op {
            RenderOpSnapshot::Newline => lines.push(Vec::new()),
            other => lines
                .last_mut()
                .expect("render plan should always have a current line")
                .push(other.clone()),
        }
    }
    lines
}

fn layout_row_containing<'a>(rows: &'a [LayoutRowSnapshot], needle: &str) -> &'a LayoutRowSnapshot {
    rows.iter()
        .find(|row| row.text.contains(needle))
        .unwrap_or_else(|| {
            panic!(
                "expected layout rows to contain text fragment {needle:?}, got rows {:?}",
                rows.iter().map(|row| row.text.as_str()).collect::<Vec<_>>()
            )
        })
}

fn first_background_bounds(row: &LayoutRowSnapshot) -> Option<(usize, usize)> {
    row.background_runs
        .first()
        .map(|run| (run.start_column, run.end_column))
}

fn background_bounds_at_level(
    row: &LayoutRowSnapshot,
    visual_level: usize,
) -> Option<(usize, usize)> {
    row.background_runs
        .iter()
        .find(|run| run.visual_level == visual_level)
        .map(|run| (run.start_column, run.end_column))
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn relative_luminance(color: RgbColor) -> f32 {
    0.2126 * color.0 as f32 + 0.7152 * color.1 as f32 + 0.0722 * color.2 as f32
}

fn fixture_path(relative_path: &str) -> PathBuf {
    Path::new("testdata/fixtures").join(relative_path)
}

fn fixture_family(relative_path: &str) -> &str {
    relative_path
        .split('/')
        .next()
        .expect("fixture path should have a top-level family directory")
}

fn showcase_family(path: &Path) -> &str {
    path.strip_prefix("testdata/showcase")
        .expect("showcase path should live under testdata/showcase")
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .expect("showcase path should have a top-level family directory")
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn collect_files(root: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_recursive(Path::new(root), &mut files);
    files.sort();
    files
}

fn collect_files_recursive(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read directory {}: {error}", root.display()))
    {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", root.display()));
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursive(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn is_supported_highlight_path(path: &Path) -> bool {
    detected_runtime(Some(path), "").is_some()
}

fn detected_runtime(path: Option<&Path>, source: &str) -> Option<RuntimeId> {
    detect_document_kind(path, source).map(DocumentKind::runtime_id)
}
