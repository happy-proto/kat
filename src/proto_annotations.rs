use std::ops::Range;

use tree_sitter::Node;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AnnotationStringKind {
    Cel,
    HttpPath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AnnotationString {
    pub(crate) kind: AnnotationStringKind,
    pub(crate) content: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathCapture {
    pub(crate) range: Range<usize>,
    pub(crate) capture: &'static str,
}

pub(crate) fn annotation_strings(root: Node<'_>, source: &str) -> Vec<AnnotationString> {
    let mut strings = Vec::new();
    visit_options(root, source, &mut strings);
    strings
}

fn visit_options(node: Node<'_>, source: &str, strings: &mut Vec<AnnotationString>) {
    if matches!(node.kind(), "option" | "field_option") {
        let mut cursor = node.walk();
        let mut children = node.named_children(&mut cursor);
        if let Some(name) = children.find(|child| child.kind() == "full_ident") {
            let extension = source[name.byte_range()].trim_start_matches('.');
            let kind = match extension {
                "google.api.http" => Some(AnnotationStringKind::HttpPath),
                "buf.validate.field" | "buf.validate.message" => Some(AnnotationStringKind::Cel),
                _ => None,
            };
            if let Some(kind) = kind {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if child.kind() == "constant" {
                        visit_option_value(child, source, kind, None, strings);
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_options(child, source, strings);
    }
}

fn visit_option_value(
    node: Node<'_>,
    source: &str,
    kind: AnnotationStringKind,
    key: Option<&str>,
    strings: &mut Vec<AnnotationString>,
) {
    if node.kind() == "string" {
        let wanted = match kind {
            AnnotationStringKind::Cel => key == Some("expression"),
            AnnotationStringKind::HttpPath => {
                matches!(
                    key,
                    Some("get" | "put" | "post" | "delete" | "patch" | "path")
                )
            }
        };
        if wanted && let Some(content) = quoted_content_range(node, source) {
            strings.push(AnnotationString { kind, content });
        }
        return;
    }

    if node.kind() == "block_lit" {
        let mut cursor = node.walk();
        let mut children = node.named_children(&mut cursor);
        while let Some(child) = children.next() {
            if child.kind() != "identifier" {
                visit_option_value(child, source, kind, None, strings);
                continue;
            }
            let Some(value) = children.next() else {
                break;
            };
            let key = &source[child.byte_range()];
            visit_option_value(value, source, kind, Some(key), strings);
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_option_value(child, source, kind, key, strings);
    }
}

fn quoted_content_range(node: Node<'_>, source: &str) -> Option<Range<usize>> {
    let range = node.byte_range();
    let bytes = source[range.clone()].as_bytes();
    if bytes.len() >= 2 && matches!(bytes[0], b'\'' | b'"') && bytes[0] == bytes[bytes.len() - 1] {
        Some((range.start + 1)..(range.end - 1))
    } else {
        None
    }
}

pub(crate) fn http_path_captures(source: &str, range: Range<usize>) -> Vec<PathCapture> {
    let bytes = source[range.clone()].as_bytes();
    let mut spans = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' {
            cursor = (cursor + 2).min(bytes.len());
            continue;
        }
        if bytes[cursor] != b'{' {
            cursor += 1;
            continue;
        }
        let open = cursor;
        let Some(close_offset) = bytes[(open + 1)..].iter().position(|byte| *byte == b'}') else {
            break;
        };
        let close = open + 1 + close_offset;
        spans.push(path_capture(
            range.start + open..range.start + open + 1,
            "punctuation.bracket",
        ));
        spans.push(path_capture(
            range.start + close..range.start + close + 1,
            "punctuation.bracket",
        ));
        let body = &bytes[(open + 1)..close];
        let equal = body.iter().position(|byte| *byte == b'=');
        let field_end = equal.map_or(close, |offset| open + 1 + offset);
        if field_end > open + 1 {
            spans.push(path_capture(
                range.start + open + 1..range.start + field_end,
                "variable.parameter",
            ));
        }
        if let Some(equal) = equal {
            let equal = open + 1 + equal;
            spans.push(path_capture(
                range.start + equal..range.start + equal + 1,
                "operator",
            ));
            for (index, byte) in bytes.iter().enumerate().take(close).skip(equal + 1) {
                if *byte == b'*' {
                    spans.push(path_capture(
                        range.start + index..range.start + index + 1,
                        "operator",
                    ));
                }
            }
        }
        cursor = close + 1;
    }

    if let Some(colon) = bytes.iter().rposition(|byte| *byte == b':')
        && bytes[(colon + 1)..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        && colon + 1 < bytes.len()
    {
        spans.push(path_capture(
            range.start + colon..range.start + colon + 1,
            "operator",
        ));
        spans.push(path_capture(
            range.start + colon + 1..range.end,
            "function.call",
        ));
    }
    spans
}

fn path_capture(range: Range<usize>, capture: &'static str) -> PathCapture {
    PathCapture { range, capture }
}
