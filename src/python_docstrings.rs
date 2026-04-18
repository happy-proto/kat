use std::ops::Range;

use anyhow::Result;

use crate::{
    HighlightRenderData, NestedRegion, StyledSpan, VisualRegion, build_region_segments,
    document_kind::{DocumentKind, DocumentProfile},
    host_injections::{InjectionVisualAnchor, InjectionVisualKind},
    map_virtual_nested_regions_to_source, map_virtual_regions_to_source,
    map_virtual_spans_to_source, plain_document_kind, render_virtual_injection_render_data,
    theme::Theme,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SectionKind {
    Parameters,
    Returns,
    Raises,
    Attributes,
    Examples,
    Notes,
    SeeAlso,
}

#[derive(Clone, Copy)]
struct DocstringLine<'a> {
    end: usize,
    trimmed_start: usize,
    trimmed: &'a str,
    indent: usize,
}

pub(crate) fn resolve_python_docstring_document_kind(
    document_kind: DocumentKind,
    source: &str,
) -> DocumentKind {
    if document_kind.runtime_name() != "python_docstring" {
        return document_kind;
    }

    let profile = match document_kind.profile() {
        DocumentProfile::PythonDocstringAuto => detect_docstring_profile(source),
        profile => profile,
    };

    DocumentKind::with_profile("python_docstring", profile)
}

pub(crate) fn render_python_docstring(
    document_kind: DocumentKind,
    source: &str,
    theme: &Theme,
    nested_visual_level: usize,
) -> Result<HighlightRenderData> {
    let resolved_document_kind = resolve_python_docstring_document_kind(document_kind, source);
    let lines = docstring_lines(source);
    let mut spans = Vec::new();
    let mut nested_regions = Vec::new();

    let mut google_section = None;
    let mut numpy_section = None;

    for (index, line) in lines.iter().copied().enumerate() {
        if let Some((section, underline_index)) = numpy_section_header(&lines, index) {
            highlight_numpy_section_header(&mut spans, theme, source, line, lines[underline_index]);
            numpy_section = Some((section, line.indent));
            google_section = None;
            continue;
        }

        if is_numpy_underline(&line) {
            continue;
        }

        if let Some(section) = google_section_header(line.trimmed) {
            highlight_google_section_header(&mut spans, theme, source, line);
            google_section = Some((section, line.indent));
            numpy_section = None;
        } else {
            if !line.trimmed.is_empty() {
                if let Some((_, indent)) = google_section
                    && line.indent <= indent
                {
                    google_section = None;
                }
                if let Some((_, indent)) = numpy_section
                    && line.indent <= indent
                    && !is_numpy_like_entry(line.trimmed)
                {
                    numpy_section = None;
                }
            }

            if let Some((section, _)) = google_section {
                highlight_google_section_entry(&mut spans, theme, source, line, section);
            }

            if let Some((section, _)) = numpy_section {
                highlight_numpy_section_entry(&mut spans, theme, source, line, section);
            }
        }

        highlight_rest_field_line(&mut spans, theme, source, line);
        highlight_inline_roles(&mut spans, theme, source, line);
        highlight_double_backtick_literals(&mut spans, theme, source, line);
        highlight_doctest_prompt(&mut spans, theme, source, line);
    }

    nested_regions.extend(collect_doctest_regions(source, theme, nested_visual_level)?);

    spans.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then(left.range.end.cmp(&right.range.end))
    });
    spans.dedup_by(|left, right| left.range == right.range && left.style == right.style);

    Ok(HighlightRenderData {
        resolved_document_kind,
        spans,
        nested_regions,
    })
}

fn detect_docstring_profile(source: &str) -> DocumentProfile {
    let lines = docstring_lines(source);
    let mut rest_score = 0;
    let mut google_score = 0;
    let mut numpy_score = 0;

    for (index, line) in lines.iter().enumerate() {
        if is_rest_field(line.trimmed)
            || line.trimmed.contains(":class:`")
            || line.trimmed.contains(":meth:`")
        {
            rest_score += 1;
        }
        if google_section_header(line.trimmed).is_some() {
            google_score += 1;
        }
        if numpy_section_header(&lines, index).is_some() {
            numpy_score += 1;
        }
    }

    if numpy_score >= google_score && numpy_score >= rest_score && numpy_score > 0 {
        DocumentProfile::PythonDocstringNumpy
    } else if google_score >= rest_score && google_score > 0 {
        DocumentProfile::PythonDocstringGoogle
    } else if rest_score > 0 {
        DocumentProfile::PythonDocstringRest
    } else {
        DocumentProfile::PythonDocstringPlain
    }
}

fn docstring_lines(source: &str) -> Vec<DocstringLine<'_>> {
    let mut lines = Vec::new();
    let mut line_start = 0;

    while line_start <= source.len() {
        let line_end = source[line_start..]
            .find('\n')
            .map(|offset| line_start + offset)
            .unwrap_or(source.len());
        let text = &source[line_start..line_end];
        let trimmed = text.trim_start_matches([' ', '\t']);
        let indent = text.len() - trimmed.len();
        lines.push(DocstringLine {
            end: line_end,
            trimmed_start: line_start + indent,
            trimmed,
            indent,
        });

        if line_end == source.len() {
            break;
        }

        line_start = line_end + 1;
    }

    lines
}

fn push_capture(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    range: Range<usize>,
    capture: &'static str,
) {
    if range.start >= range.end {
        return;
    }
    let Some(text) = source.get(range.clone()) else {
        return;
    };
    let Some(style) = theme.token_style_for(capture, text) else {
        return;
    };
    spans.push(StyledSpan {
        range,
        style: Some(style),
    });
}

fn is_rest_field(line: &str) -> bool {
    line.starts_with(':') && line[1..].contains(':')
}

fn highlight_rest_field_line(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    line: DocstringLine<'_>,
) {
    let Some(field_text) = line.trimmed.strip_prefix(':') else {
        return;
    };
    let Some(field_name_end) = field_text.find([' ', ':']) else {
        return;
    };
    let field_name = &field_text[..field_name_end];
    let rest = &field_text[field_name_end..];
    let Some(closing_colon_offset) = rest.find(':') else {
        return;
    };
    let field_name_start = line.trimmed_start + 1;
    let field_name_end_abs = field_name_start + field_name.len();
    let closing_colon_abs = field_name_end_abs + rest[..closing_colon_offset].len();

    push_capture(
        spans,
        theme,
        source,
        line.trimmed_start..line.trimmed_start + 1,
        "punctuation.special",
    );
    push_capture(
        spans,
        theme,
        source,
        field_name_start..field_name_end_abs,
        "keyword.directive",
    );
    push_capture(
        spans,
        theme,
        source,
        closing_colon_abs..closing_colon_abs + 1,
        "punctuation.special",
    );

    let argument_text = rest[..closing_colon_offset].trim();
    if argument_text.is_empty() {
        return;
    }

    let argument_start = line
        .trimmed
        .find(argument_text)
        .map(|offset| line.trimmed_start + offset)
        .unwrap_or(field_name_end_abs);
    let argument_end = argument_start + argument_text.len();
    let capture = match field_name {
        "param" | "parameter" | "arg" | "argument" | "type" => "variable.parameter",
        "raises" | "raise" | "exception" | "except" | "exc" | "rtype" => "type",
        _ => return,
    };
    push_capture(spans, theme, source, argument_start..argument_end, capture);
}

fn highlight_inline_roles(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    line: DocstringLine<'_>,
) {
    let mut offset = 0;
    while let Some(start_rel) = line.trimmed[offset..].find(':') {
        let start = offset + start_rel;
        let after_start = &line.trimmed[start + 1..];
        let Some(role_end_rel) = after_start.find(":`") else {
            offset = start + 1;
            continue;
        };
        let role = &after_start[..role_end_rel];
        if role.is_empty()
            || !role
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            offset = start + 1;
            continue;
        }
        let target_start = start + 1 + role_end_rel + 2;
        let Some(target_end_rel) = line.trimmed[target_start..].find('`') else {
            offset = start + 1;
            continue;
        };
        let target_end = target_start + target_end_rel;
        let absolute_start = line.trimmed_start + start;
        let absolute_role_start = absolute_start + 1;
        let absolute_role_end = absolute_role_start + role.len();
        let absolute_target_start = line.trimmed_start + target_start;
        let absolute_target_end = line.trimmed_start + target_end;

        push_capture(
            spans,
            theme,
            source,
            absolute_start..absolute_start + 1,
            "punctuation.special",
        );
        push_capture(
            spans,
            theme,
            source,
            absolute_role_start..absolute_role_end,
            "keyword.directive",
        );
        push_capture(
            spans,
            theme,
            source,
            absolute_role_end..absolute_role_end + 2,
            "punctuation.special",
        );
        push_capture(
            spans,
            theme,
            source,
            absolute_target_start..absolute_target_end,
            role_target_capture(role),
        );
        push_capture(
            spans,
            theme,
            source,
            absolute_target_end..absolute_target_end + 1,
            "punctuation.special",
        );

        offset = target_end + 1;
    }
}

fn role_target_capture(role: &str) -> &'static str {
    match role {
        "class" | "exc" | "attr" | "obj" | "type" => "type",
        "meth" | "func" => "function",
        "mod" | "ref" | "term" => "string.special",
        _ => "string.special",
    }
}

fn highlight_double_backtick_literals(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    line: DocstringLine<'_>,
) {
    let mut offset = 0;
    while let Some(start_rel) = line.trimmed[offset..].find("``") {
        let start = offset + start_rel;
        let Some(end_rel) = line.trimmed[start + 2..].find("``") else {
            break;
        };
        let end = start + 2 + end_rel;
        let absolute_start = line.trimmed_start + start;
        let absolute_end = line.trimmed_start + end;
        push_capture(
            spans,
            theme,
            source,
            absolute_start..absolute_start + 2,
            "punctuation.special",
        );
        push_capture(
            spans,
            theme,
            source,
            absolute_start + 2..absolute_end,
            "string.special",
        );
        push_capture(
            spans,
            theme,
            source,
            absolute_end..absolute_end + 2,
            "punctuation.special",
        );
        offset = end + 2;
    }
}

fn highlight_doctest_prompt(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    line: DocstringLine<'_>,
) {
    for prompt in [">>>", "..."] {
        if line.trimmed.starts_with(prompt) {
            push_capture(
                spans,
                theme,
                source,
                line.trimmed_start..line.trimmed_start + prompt.len(),
                "punctuation.special",
            );
            break;
        }
    }
}

fn collect_doctest_regions(
    source: &str,
    theme: &Theme,
    nested_visual_level: usize,
) -> Result<Vec<NestedRegion>> {
    let sessions = doctest_sessions(source);
    let mut regions = Vec::with_capacity(sessions.len());

    for session in sessions {
        let (virtual_source, source_map) = build_doctest_virtual_source(source, &session);
        if virtual_source.trim().is_empty() {
            continue;
        }

        let child_render = render_virtual_injection_render_data(
            plain_document_kind("python"),
            &virtual_source,
            false,
            theme,
            nested_visual_level.saturating_add(1),
            None,
        )?;

        regions.push(NestedRegion {
            visual_level: nested_visual_level.saturating_add(1),
            resolved_document_kind: child_render.resolved_document_kind,
            visual_kind: InjectionVisualKind::Transparent,
            layout_segments: build_region_segments(
                source,
                &session.code_ranges,
                InjectionVisualAnchor::Content,
                InjectionVisualKind::Transparent,
            ),
            child_regions: map_virtual_regions_to_source(
                source,
                &collect_child_visual_regions(&child_render.nested_regions),
                &source_map,
            ),
            child_nested_regions: map_virtual_nested_regions_to_source(
                source,
                &child_render.nested_regions,
                &source_map,
            ),
            overlays: map_virtual_spans_to_source(&child_render.spans, &source_map),
            merge_parent_styles: false,
        });
    }

    Ok(regions)
}

#[derive(Default)]
struct DoctestSession {
    code_ranges: Vec<Range<usize>>,
}

fn doctest_sessions(source: &str) -> Vec<DoctestSession> {
    let lines = docstring_lines(source);
    let mut sessions = Vec::new();
    let mut current = DoctestSession::default();

    for line in lines {
        if let Some(code_range) = doctest_code_range(line) {
            current.code_ranges.push(code_range);
        } else if !current.code_ranges.is_empty() {
            sessions.push(std::mem::take(&mut current));
        }
    }

    if !current.code_ranges.is_empty() {
        sessions.push(current);
    }

    sessions
}

fn doctest_code_range(line: DocstringLine<'_>) -> Option<Range<usize>> {
    for prompt in [">>> ", "... "] {
        if line.trimmed.starts_with(prompt) {
            let code_start = line.trimmed_start + prompt.len();
            return Some(code_start..line.end);
        }
    }
    None
}

fn build_doctest_virtual_source(
    source: &str,
    session: &DoctestSession,
) -> (String, Vec<Range<usize>>) {
    let mut virtual_source = String::new();
    let mut source_map = Vec::new();

    for (index, range) in session.code_ranges.iter().enumerate() {
        let slice = &source[range.clone()];
        virtual_source.push_str(slice);
        for offset in 0..slice.len() {
            source_map.push((range.start + offset)..(range.start + offset + 1));
        }

        if index + 1 < session.code_ranges.len() {
            virtual_source.push('\n');
            source_map.push(0..0);
        }
    }

    (virtual_source, source_map)
}

fn collect_child_visual_regions(nested_regions: &[NestedRegion]) -> Vec<VisualRegion> {
    nested_regions
        .iter()
        .map(|region| VisualRegion {
            visual_kind: region.visual_kind,
            visual_level: region.visual_level,
            segments: region.layout_segments.clone(),
        })
        .collect()
}

fn google_section_header(line: &str) -> Option<SectionKind> {
    match line {
        "Args:" | "Arguments:" => Some(SectionKind::Parameters),
        "Returns:" | "Yields:" => Some(SectionKind::Returns),
        "Raises:" => Some(SectionKind::Raises),
        "Attributes:" => Some(SectionKind::Attributes),
        "Example:" | "Examples:" => Some(SectionKind::Examples),
        "Note:" | "Notes:" => Some(SectionKind::Notes),
        "See Also:" => Some(SectionKind::SeeAlso),
        _ => None,
    }
}

fn highlight_google_section_header(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    line: DocstringLine<'_>,
) {
    let name = &line.trimmed[..line.trimmed.len() - 1];
    push_capture(
        spans,
        theme,
        source,
        line.trimmed_start..line.trimmed_start + name.len(),
        "keyword.directive",
    );
    push_capture(
        spans,
        theme,
        source,
        line.end - 1..line.end,
        "punctuation.special",
    );
}

fn highlight_google_section_entry(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    line: DocstringLine<'_>,
    section: SectionKind,
) {
    let capture = match section {
        SectionKind::Parameters | SectionKind::Attributes => Some("variable.parameter"),
        SectionKind::Raises => Some("type"),
        SectionKind::SeeAlso => Some("function"),
        _ => None,
    };

    if let Some(capture) = capture
        && let Some(colon) = line.trimmed.find(':')
        && colon > 0
    {
        push_capture(
            spans,
            theme,
            source,
            line.trimmed_start..line.trimmed_start + colon,
            capture,
        );
        push_capture(
            spans,
            theme,
            source,
            line.trimmed_start + colon..line.trimmed_start + colon + 1,
            "punctuation.special",
        );
    } else if section == SectionKind::Returns
        && !line.trimmed.is_empty()
        && !line.trimmed.starts_with('\'')
    {
        push_capture(
            spans,
            theme,
            source,
            line.trimmed_start..line.trimmed_start + line.trimmed.len(),
            "type",
        );
    }
}

fn numpy_section_header(lines: &[DocstringLine<'_>], index: usize) -> Option<(SectionKind, usize)> {
    let line = lines[index];
    let next = lines.get(index + 1).copied()?;
    let section = match line.trimmed {
        "Parameters" | "Other Parameters" => SectionKind::Parameters,
        "Returns" | "Yields" => SectionKind::Returns,
        "Raises" => SectionKind::Raises,
        "Attributes" => SectionKind::Attributes,
        "Examples" => SectionKind::Examples,
        "Notes" => SectionKind::Notes,
        "See Also" => SectionKind::SeeAlso,
        _ => return None,
    };

    is_numpy_underline(&next).then_some((section, index + 1))
}

fn is_numpy_underline(line: &DocstringLine<'_>) -> bool {
    let trimmed = line.trimmed;
    trimmed.len() >= 3 && trimmed.chars().all(|ch| ch == '-' || ch == '=')
}

fn highlight_numpy_section_header(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    header: DocstringLine<'_>,
    underline: DocstringLine<'_>,
) {
    push_capture(
        spans,
        theme,
        source,
        header.trimmed_start..header.trimmed_start + header.trimmed.len(),
        "keyword.directive",
    );
    push_capture(
        spans,
        theme,
        source,
        underline.trimmed_start..underline.trimmed_start + underline.trimmed.len(),
        "punctuation.special",
    );
}

fn highlight_numpy_section_entry(
    spans: &mut Vec<StyledSpan>,
    theme: &Theme,
    source: &str,
    line: DocstringLine<'_>,
    section: SectionKind,
) {
    if line.trimmed.is_empty() {
        return;
    }

    if let Some(colon) = line.trimmed.find(" : ") {
        let name_end = line.trimmed_start + colon;
        let type_start = name_end + 3;
        let type_end = line.trimmed_start + line.trimmed.len();
        let capture = match section {
            SectionKind::Parameters | SectionKind::Attributes => "variable.parameter",
            SectionKind::Raises => "type",
            _ => "variable.parameter",
        };
        push_capture(spans, theme, source, line.trimmed_start..name_end, capture);
        push_capture(
            spans,
            theme,
            source,
            name_end + 1..name_end + 2,
            "punctuation.special",
        );
        push_capture(spans, theme, source, type_start..type_end, "type");
        return;
    }

    match section {
        SectionKind::Returns => push_capture(
            spans,
            theme,
            source,
            line.trimmed_start..line.trimmed_start + line.trimmed.len(),
            "type",
        ),
        SectionKind::SeeAlso => push_capture(
            spans,
            theme,
            source,
            line.trimmed_start..line.trimmed_start + line.trimmed.len(),
            "function",
        ),
        _ => {}
    }
}

fn is_numpy_like_entry(line: &str) -> bool {
    line.contains(" : ")
        || line
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '[' | ']'))
}
