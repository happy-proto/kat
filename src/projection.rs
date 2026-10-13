use crate::host_injections::InjectionCandidate;
use crate::host_injections::InjectionDecode;
use crate::host_injections::InjectionProjection;
use crate::host_injections::InjectionProjectionSegment;
use crate::injection::line_start_offset;
use crate::render_data::NestedRegion;
use crate::render_data::RegionSegment;
use crate::render_data::StyledSpan;
use crate::render_data::VisualRegion;
use crate::source_map::SourceMap;
use crate::span_ops::push_span;
use std::ops::Range;

pub(crate) fn build_virtual_source(
    source: &str,
    ranges: &[Range<usize>],
    strip_shared_indent: bool,
    decode: InjectionDecode,
) -> (String, SourceMap) {
    let mut virtual_source = String::new();
    let mut source_map = SourceMap::default();
    let shared_indent = if strip_shared_indent {
        shared_leading_indent(source, ranges)
    } else {
        0
    };

    for (index, range) in ranges.iter().enumerate() {
        if index > 0 && !virtual_source.ends_with('\n') {
            virtual_source.push('\n');
            source_map.synthetic(1);
        }

        let source_slice = &source[range.clone()];
        let trim_len = source_slice
            .chars()
            .take_while(|ch| matches!(ch, ' ' | '\t'))
            .map(char::len_utf8)
            .take(shared_indent)
            .sum::<usize>();
        let trimmed_range = (range.start + trim_len)..range.end;

        append_injection_content(
            source,
            trimmed_range,
            decode,
            &mut virtual_source,
            &mut source_map,
        );
    }

    (virtual_source, source_map)
}

pub(crate) fn build_candidate_virtual_source(
    source: &str,
    candidate: &InjectionCandidate,
) -> (String, SourceMap) {
    match &candidate.projection {
        InjectionProjection::RawRanges => build_virtual_source(
            source,
            &candidate.ranges,
            candidate.strip_shared_indent,
            candidate.decode,
        ),
        InjectionProjection::TemplateSegments(segments) => build_projected_virtual_source(
            source,
            &candidate.ranges,
            segments,
            candidate.strip_shared_indent,
        ),
    }
}

pub(crate) fn build_projected_virtual_source(
    source: &str,
    ranges: &[Range<usize>],
    segments: &[InjectionProjectionSegment],
    strip_shared_indent: bool,
) -> (String, SourceMap) {
    let shared_indent = if strip_shared_indent {
        shared_leading_indent(source, ranges)
    } else {
        0
    };
    let mut virtual_source = String::new();
    let mut source_map = SourceMap::default();
    let mut at_line_start = true;
    let mut trim_remaining = shared_indent;

    for segment in segments {
        match segment {
            InjectionProjectionSegment::Source(range) => append_projected_source_range(
                source,
                range,
                shared_indent,
                &mut virtual_source,
                &mut source_map,
                &mut at_line_start,
                &mut trim_remaining,
            ),
            InjectionProjectionSegment::Synthetic(text) => append_projected_synthetic_text(
                text,
                &mut virtual_source,
                &mut source_map,
                &mut at_line_start,
                &mut trim_remaining,
                shared_indent,
            ),
            InjectionProjectionSegment::MappedText { text, source } => {
                append_mapped_text(&mut virtual_source, &mut source_map, source.clone(), text);
                if !text.is_empty() {
                    at_line_start = text.ends_with('\n');
                    trim_remaining = if at_line_start { shared_indent } else { 0 };
                }
            }
        }
    }

    (virtual_source, source_map)
}

pub(crate) fn append_projected_source_range(
    source: &str,
    range: &Range<usize>,
    shared_indent: usize,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
    at_line_start: &mut bool,
    trim_remaining: &mut usize,
) {
    let mut cursor = range.start;
    while cursor < range.end {
        let ch = source[cursor..]
            .chars()
            .next()
            .expect("source range should stay on character boundary");
        let next = cursor + ch.len_utf8();
        if *at_line_start && *trim_remaining > 0 && matches!(ch, ' ' | '\t') {
            *trim_remaining -= 1;
            cursor = next;
            continue;
        }

        append_raw_range(source, cursor..next, virtual_source, source_map);
        cursor = next;
        if ch == '\n' {
            *at_line_start = true;
            *trim_remaining = shared_indent;
        } else {
            *at_line_start = false;
            *trim_remaining = 0;
        }
    }
}

pub(crate) fn append_projected_synthetic_text(
    text: &str,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
    at_line_start: &mut bool,
    trim_remaining: &mut usize,
    shared_indent: usize,
) {
    virtual_source.push_str(text);
    source_map.synthetic(text.len());

    if let Some(last_newline) = text.rfind('\n') {
        *at_line_start = true;
        let trailing = &text[last_newline + 1..];
        let consumed_indent = trailing
            .chars()
            .take_while(|ch| matches!(ch, ' ' | '\t'))
            .count();
        *trim_remaining = shared_indent.saturating_sub(consumed_indent);
    } else if !text.is_empty() {
        *at_line_start = false;
        *trim_remaining = 0;
    }
}

pub(crate) fn append_injection_content(
    source: &str,
    range: Range<usize>,
    decode: InjectionDecode,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    match decode {
        InjectionDecode::None => append_raw_range(source, range, virtual_source, source_map),
        InjectionDecode::JavaScriptLiteral => {
            decode_javascript_string_literal(source, range, virtual_source, source_map)
        }
        InjectionDecode::JavaScriptString => {
            decode_javascript_string_content(source, range, virtual_source, source_map)
        }
        InjectionDecode::PythonLiteral => {
            decode_python_string_literal(source, range, virtual_source, source_map)
        }
        InjectionDecode::PythonString => {
            decode_python_string_content(source, range, virtual_source, source_map)
        }
        InjectionDecode::RustString => {
            decode_rust_string_content(source, range, virtual_source, source_map)
        }
        InjectionDecode::GoString => {
            decode_go_string_content(source, range, virtual_source, source_map)
        }
        InjectionDecode::ProtoString => decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::Proto,
            virtual_source,
            source_map,
        ),
    }
}

pub(crate) fn append_raw_range(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    let slice = &source[range.clone()];
    virtual_source.push_str(slice);
    source_map.identity(range);
}

pub(crate) fn append_mapped_text(
    virtual_source: &mut String,
    source_map: &mut SourceMap,
    source_range: Range<usize>,
    text: &str,
) {
    virtual_source.push_str(text);
    source_map.decoded(text.len(), source_range);
}

pub(crate) fn decode_javascript_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    match surrounding_delimiter(source, &range) {
        Some('\'') | Some('"') => decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::JavaScript,
            virtual_source,
            source_map,
        ),
        _ => append_raw_range(source, range, virtual_source, source_map),
    }
}

pub(crate) fn decode_javascript_string_literal(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    let slice = &source[range.clone()];
    if slice.len() >= 2
        && matches!(slice.as_bytes().first(), Some(b'\'') | Some(b'"'))
        && slice.as_bytes().first() == slice.as_bytes().last()
    {
        decode_javascript_string_content(
            source,
            (range.start + 1)..(range.end - 1),
            virtual_source,
            source_map,
        );
    } else {
        append_raw_range(source, range, virtual_source, source_map);
    }
}

pub(crate) fn decode_python_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    if python_string_is_raw(source, &range) {
        append_raw_range(source, range, virtual_source, source_map);
    } else {
        decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::Python,
            virtual_source,
            source_map,
        );
    }
}

pub(crate) fn decode_python_string_literal(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    let slice = &source[range.clone()];
    let Some((content_start, content_end)) = python_string_literal_content_bounds(slice) else {
        append_raw_range(source, range, virtual_source, source_map);
        return;
    };

    let content_range = (range.start + content_start)..(range.start + content_end);
    decode_python_string_content(source, content_range, virtual_source, source_map);
}

pub(crate) fn decode_rust_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    if rust_string_is_raw(source, &range) {
        append_raw_range(source, range, virtual_source, source_map);
    } else {
        decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::Rust,
            virtual_source,
            source_map,
        );
    }
}

pub(crate) fn decode_go_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    match surrounding_delimiter(source, &range) {
        Some('`') => append_raw_range(source, range, virtual_source, source_map),
        Some('"') => decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::Go,
            virtual_source,
            source_map,
        ),
        _ => append_raw_range(source, range, virtual_source, source_map),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecodeFlavor {
    JavaScript,
    Python,
    Rust,
    Go,
    Proto,
}

pub(crate) fn decode_backslash_escaped_range(
    source: &str,
    range: Range<usize>,
    flavor: DecodeFlavor,
    virtual_source: &mut String,
    source_map: &mut SourceMap,
) {
    let slice = &source[range.clone()];
    let bytes = slice.as_bytes();
    let mut cursor = 0;
    let mut plain_start = 0;

    while cursor < bytes.len() {
        if bytes[cursor] != b'\\' {
            cursor += 1;
            continue;
        }

        if plain_start < cursor {
            append_raw_range(
                source,
                (range.start + plain_start)..(range.start + cursor),
                virtual_source,
                source_map,
            );
        }

        let escape_start = cursor;
        cursor += 1;
        if cursor >= bytes.len() {
            append_mapped_text(
                virtual_source,
                source_map,
                (range.start + escape_start)..(range.start + cursor),
                "\\",
            );
            break;
        }

        let next = bytes[cursor] as char;
        let decoded = match next {
            '\\' => Some(("\\".to_owned(), 1)),
            '\'' => Some(("\'".to_owned(), 1)),
            '"' => Some(("\"".to_owned(), 1)),
            'n' => Some(("\n".to_owned(), 1)),
            'r' => Some(("\r".to_owned(), 1)),
            't' => Some(("\t".to_owned(), 1)),
            'b' => Some(("\u{0008}".to_owned(), 1)),
            'f' => Some(("\u{000C}".to_owned(), 1)),
            'v' if matches!(
                flavor,
                DecodeFlavor::JavaScript | DecodeFlavor::Go | DecodeFlavor::Proto
            ) =>
            {
                Some(("\u{000B}".to_owned(), 1))
            }
            '0' => Some(("\0".to_owned(), 1)),
            'x' => decode_fixed_width_hex(bytes, cursor + 1, 2),
            'u' => match flavor {
                DecodeFlavor::Rust | DecodeFlavor::JavaScript
                    if bytes.get(cursor + 1) == Some(&b'{') =>
                {
                    decode_braced_hex(bytes, cursor + 2)
                }
                DecodeFlavor::Go if bytes.get(cursor + 1) == Some(&b'{') => {
                    decode_braced_hex(bytes, cursor + 2)
                }
                _ => decode_fixed_width_hex(bytes, cursor + 1, 4),
            },
            'U' if flavor == DecodeFlavor::Python => decode_fixed_width_hex(bytes, cursor + 1, 8),
            '\n' if matches!(flavor, DecodeFlavor::JavaScript | DecodeFlavor::Python) => {
                Some((String::new(), 1))
            }
            _ => None,
        };

        if let Some((text, consumed_after_marker)) = decoded {
            let consumed = 1 + consumed_after_marker;
            let escape_range =
                (range.start + escape_start)..(range.start + escape_start + consumed);
            append_mapped_text(virtual_source, source_map, escape_range, &text);
            cursor = escape_start + consumed;
        } else {
            append_raw_range(
                source,
                (range.start + escape_start)..(range.start + escape_start + 2),
                virtual_source,
                source_map,
            );
            cursor = escape_start + 2;
        }

        plain_start = cursor;
    }

    if plain_start < bytes.len() {
        append_raw_range(
            source,
            (range.start + plain_start)..range.end,
            virtual_source,
            source_map,
        );
    }
}

pub(crate) fn decode_fixed_width_hex(
    bytes: &[u8],
    start: usize,
    width: usize,
) -> Option<(String, usize)> {
    let digits = bytes.get(start..start + width)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }

    let value = u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()?;
    let ch = char::from_u32(value)?;
    Some((ch.to_string(), 1 + width))
}

pub(crate) fn decode_braced_hex(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    let rest = bytes.get(start..)?;
    let close_offset = rest.iter().position(|byte| *byte == b'}')?;
    let digits = &rest[..close_offset];
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }

    let value = u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()?;
    let ch = char::from_u32(value)?;
    Some((ch.to_string(), close_offset + 2))
}

pub(crate) fn surrounding_delimiter(source: &str, range: &Range<usize>) -> Option<char> {
    source[..range.start].chars().next_back()
}

pub(crate) fn python_string_is_raw(source: &str, range: &Range<usize>) -> bool {
    let prefix = literal_prefix(source, range, &['"', '\'']);
    prefix.chars().any(|ch| matches!(ch, 'r' | 'R'))
}

pub(crate) fn python_string_literal_content_bounds(literal: &str) -> Option<(usize, usize)> {
    let bytes = literal.as_bytes();
    let prefix_len = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_alphabetic())
        .count();
    let quote = *bytes.get(prefix_len)?;
    if !matches!(quote, b'"' | b'\'') {
        return None;
    }

    let quote_width = if bytes.get(prefix_len..prefix_len + 3) == Some(&[quote, quote, quote]) {
        3
    } else {
        1
    };
    let content_start = prefix_len + quote_width;
    let content_end = literal.len().checked_sub(quote_width)?;
    (content_start <= content_end).then_some((content_start, content_end))
}

pub(crate) fn rust_string_is_raw(source: &str, range: &Range<usize>) -> bool {
    let prefix = literal_prefix(source, range, &['"']);
    prefix.chars().any(|ch| matches!(ch, 'r' | 'R'))
}

pub(crate) fn literal_prefix(source: &str, range: &Range<usize>, delimiters: &[char]) -> String {
    let mut prefix = String::new();
    let mut cursor = range.start;

    while cursor > 0 {
        let previous = source[..cursor].chars().next_back().unwrap_or_default();
        if previous.is_ascii_alphabetic() || previous == '#' || delimiters.contains(&previous) {
            prefix.insert(0, previous);
            cursor -= previous.len_utf8();
            continue;
        }
        break;
    }

    prefix
}

pub(crate) fn shared_leading_indent(source: &str, ranges: &[Range<usize>]) -> usize {
    ranges
        .iter()
        .filter_map(|range| {
            let text = &source[range.clone()];
            if text.trim().is_empty() {
                return None;
            }

            Some(
                text.chars()
                    .take_while(|ch| matches!(ch, ' ' | '\t'))
                    .count(),
            )
        })
        .min()
        .unwrap_or(0)
}

pub(crate) fn map_virtual_spans_to_source(
    virtual_spans: &[StyledSpan],
    source_map: &SourceMap,
) -> Vec<StyledSpan> {
    let mut mapped = Vec::new();

    for span in virtual_spans {
        for range in source_map.ranges(span.range.clone()) {
            push_span(&mut mapped, range, span.style, span.hyperlink.clone());
        }
    }

    mapped
}

pub(crate) fn map_virtual_regions_to_source(
    source: &str,
    virtual_regions: &[VisualRegion],
    source_map: &SourceMap,
) -> Vec<VisualRegion> {
    let mut mapped = Vec::new();

    for region in virtual_regions {
        let segments = map_virtual_region_segments_to_source(source, &region.segments, source_map);
        mapped.push(VisualRegion {
            visual_kind: region.visual_kind,
            visual_level: region.visual_level,
            segments,
        });
    }

    mapped.sort_by_key(|region| {
        region
            .segments
            .first()
            .map(|segment| (segment.line_start, segment.left, region.visual_level))
            .unwrap_or((0, 0, region.visual_level))
    });
    mapped
}

pub(crate) fn map_virtual_nested_regions_to_source(
    source: &str,
    virtual_regions: &[NestedRegion],
    source_map: &SourceMap,
) -> Vec<NestedRegion> {
    let mut mapped = Vec::with_capacity(virtual_regions.len());

    for region in virtual_regions {
        mapped.push(NestedRegion {
            visual_level: region.visual_level,
            resolved_document_kind: region.resolved_document_kind,
            visual_kind: region.visual_kind,
            layout_segments: map_virtual_region_segments_to_source(
                source,
                &region.layout_segments,
                source_map,
            ),
            child_regions: map_virtual_regions_to_source(source, &region.child_regions, source_map),
            child_nested_regions: map_virtual_nested_regions_to_source(
                source,
                &region.child_nested_regions,
                source_map,
            ),
            overlays: map_virtual_spans_to_source(&region.overlays, source_map),
            merge_parent_styles: region.merge_parent_styles,
        });
    }

    mapped.sort_by_key(|region| {
        region
            .layout_segments
            .first()
            .map(|segment| (segment.line_start, segment.left, region.visual_level))
            .unwrap_or((0, 0, region.visual_level))
    });
    mapped
}

pub(crate) fn map_virtual_region_segments_to_source(
    source: &str,
    virtual_segments: &[RegionSegment],
    source_map: &SourceMap,
) -> Vec<RegionSegment> {
    let mut segments = Vec::new();
    for segment in virtual_segments {
        let ranges = if segment.left < segment.text_end {
            source_map.ranges(segment.left..segment.text_end)
        } else {
            source_map
                .anchor(segment.left)
                .into_iter()
                .map(|point| point..point)
                .collect()
        };
        for range in ranges {
            segments.push(RegionSegment {
                line_start: line_start_offset(source, range.start),
                left: range.start,
                text_end: range.end,
                left_column_override: segment.left_column_override,
            });
        }
    }
    segments
}
