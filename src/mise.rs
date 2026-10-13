use std::{ops::Range, path::Path};

use tree_sitter::{Node, Tree};

use crate::document_kind::DocumentKind;
use crate::host_injections::InjectionCandidate;
use crate::host_injections::InjectionDecode;
use crate::host_injections::InjectionProjection;
use crate::host_injections::InjectionProjectionSegment as Segment;
use crate::host_injections::InjectionVisualKind;
use crate::language_aliases::normalize_language_name;
use crate::language_aliases::shebang_interpreter_name;

// Generic config.toml is only special inside a known mise directory.
pub(crate) fn is_config_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let named_config = |name: &str, stem: &str| {
        name == format!("{stem}.toml")
            || name
                .strip_prefix(&format!("{stem}."))
                .and_then(|suffix| suffix.strip_suffix(".toml"))
                .is_some_and(|middle| {
                    !middle.is_empty() && middle.split('.').all(|part| !part.is_empty())
                })
    };
    if named_config(name.strip_prefix('.').unwrap_or(name), "mise") {
        return true;
    }
    let parent = path.parent();
    let mise_dir = |path: &Path| {
        matches!(
            path.file_name().and_then(|n| n.to_str()),
            Some("mise" | ".mise")
        )
    };
    if parent.is_some_and(mise_dir) && named_config(name, "config") {
        return true;
    }
    path.extension().is_some_and(|ext| ext == "toml")
        && parent.is_some_and(|parent| {
            parent.file_name().is_some_and(|n| n == "conf.d")
                && parent.parent().is_some_and(mise_dir)
        })
}

struct Entry<'a> {
    path: Vec<String>,
    value: Node<'a>,
}

type MappedScalar = (char, Range<usize>);
type DecodedString = (Range<usize>, Vec<MappedScalar>);

pub(crate) fn injections(tree: &Tree, source: &str) -> Vec<InjectionCandidate> {
    let mut entries = Vec::new();
    collect_entries(tree.root_node(), &[], source, &mut entries);
    let mut result = Vec::new();
    for entry in &entries {
        let path = &entry.path;
        if !matches!(
            path.first().map(String::as_str),
            Some("tasks" | "task_templates")
        ) {
            continue;
        }
        let (task_path, windows) = match path.len() {
            2 if path[0] == "tasks" => (&path[..], false), // command shorthand
            3 if matches!(path[2].as_str(), "run" | "run_windows") => {
                (&path[..2], path[2] == "run_windows")
            }
            _ => continue,
        };
        // Array tables and nested env/vars/task-call objects are never command sinks.
        let mut strings = Vec::new();
        match entry.value.kind() {
            "string" => strings.push(entry.value),
            "array" => strings.extend(children(entry.value).filter(|node| node.kind() == "string")),
            _ => continue,
        }
        for string in strings {
            let Some((range, units)) = decode_string(string, source) else {
                continue;
            };
            let decoded: String = units.iter().map(|(ch, _)| ch).collect();
            let language = if decoded.starts_with("#!") {
                decoded
                    .lines()
                    .next()
                    .and_then(shebang_interpreter_name)
                    .and_then(interpreter)
            } else {
                task_shell(&entries, task_path, source, &mut Vec::new())
                    .unwrap_or(Some(if windows { "batch" } else { "bash" }))
            };
            let Some(language) = language else { continue };
            if decoded.is_empty() {
                continue;
            }
            result.push(InjectionCandidate {
                document_kind: DocumentKind::plain(language),
                ranges: vec![range],
                projection: InjectionProjection::TemplateSegments(project_units(
                    &units, &decoded, source,
                )),
                is_combined: false,
                strip_shared_indent: false,
                merge_parent_styles: false,
                decode: InjectionDecode::None,
                highlight_github_expressions: false,
                visual_kind: Some(if decoded.contains('\n') {
                    InjectionVisualKind::RectBlock
                } else {
                    InjectionVisualKind::Transparent
                }),
                visual_level_bump: None,
                visual_anchor: None,
            });
        }
    }
    result
}

fn children(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .collect::<Vec<_>>()
        .into_iter()
}

fn key_parts(node: Node<'_>, source: &str) -> Option<Vec<String>> {
    match node.kind() {
        "bare_key" => Some(vec![source[node.byte_range()].to_owned()]),
        "quoted_key" => {
            let text = format!("{} = 0", &source[node.byte_range()]);
            let value: toml::Table = toml::from_str(&text).ok()?;
            Some(vec![value.keys().next()?.clone()])
        }
        "dotted_key" => {
            let mut parts = Vec::new();
            for child in children(node) {
                parts.extend(key_parts(child, source)?);
            }
            Some(parts)
        }
        _ => None,
    }
}

fn collect_entries<'a>(
    node: Node<'a>,
    prefix: &[String],
    source: &str,
    entries: &mut Vec<Entry<'a>>,
) {
    match node.kind() {
        "document" | "inline_table" => {
            for child in children(node) {
                collect_entries(child, prefix, source, entries);
            }
        }
        "table" => {
            let mut nodes = children(node).filter(|node| node.kind() != "comment");
            let Some(path) = nodes.next().and_then(|key| key_parts(key, source)) else {
                return;
            };
            for pair in nodes {
                collect_entries(pair, &path, source, entries);
            }
        }
        "pair" => {
            let mut nodes = children(node);
            let Some(parts) = nodes.next().and_then(|key| key_parts(key, source)) else {
                return;
            };
            let Some(value) = nodes.next() else { return };
            let mut path = prefix.to_vec();
            path.extend(parts);
            if value.kind() == "inline_table" {
                collect_entries(value, &path, source, entries);
            } else {
                entries.push(Entry { path, value });
            }
        }
        _ => {}
    }
}

fn lookup<'a>(entries: &'a [Entry<'_>], path: &[String]) -> Option<&'a Entry<'a>> {
    entries.iter().find(|entry| entry.path == path)
}

fn string_value(entry: &Entry<'_>, source: &str) -> Option<String> {
    let (_, units) = decode_string(entry.value, source)?;
    Some(units.into_iter().map(|(ch, _)| ch).collect())
}

// Some(None) means a declared but unknown/dynamic interpreter: do not guess Bash.
fn task_shell(
    entries: &[Entry<'_>],
    task: &[String],
    source: &str,
    visited: &mut Vec<Vec<String>>,
) -> Option<Option<&'static str>> {
    if visited.iter().any(|path| path == task) {
        return Some(None);
    }
    visited.push(task.to_vec());
    let mut path = task.to_vec();
    path.push("shell".to_owned());
    if let Some(entry) = lookup(entries, &path) {
        return Some(string_value(entry, source).as_deref().and_then(interpreter));
    }
    *path.last_mut()? = "extends".to_owned();
    if let Some(entry) = lookup(entries, &path) {
        let name = string_value(entry, source)?;
        let template = vec!["task_templates".to_owned(), name];
        if !entries
            .iter()
            .any(|entry| entry.path.starts_with(&template))
        {
            return Some(None);
        }
        if let Some(shell) = task_shell(entries, &template, source, visited) {
            return Some(shell);
        }
    }
    let path = vec!["task_config".to_owned(), "shell".to_owned()];
    lookup(entries, &path).map(|entry| string_value(entry, source).as_deref().and_then(interpreter))
}

fn interpreter(command: &str) -> Option<&'static str> {
    if command.contains("{{") || command.contains("{%") {
        return None;
    }
    match normalize_language_name(command)? {
        "bash" => Some("bash"),
        "zsh" => Some("zsh"),
        "fish" => Some("fish"),
        "powershell" => Some("powershell"),
        "batch" => Some("batch"),
        "python" => Some("python"),
        "javascript" | "deno" => Some("javascript"),
        "ruby" => Some("ruby"),
        "perl" => Some("perl"),
        _ => None,
    }
}

// Decode TOML while retaining a source range for each Unicode scalar. Escapes map
// back to their entire spelling; trimmed newlines/continuations have no child style.
fn decode_string(node: Node<'_>, source: &str) -> Option<DecodedString> {
    if node.kind() != "string" || node.has_error() {
        return None;
    }
    let text = &source[node.byte_range()];
    let quote = text.chars().next()?;
    let width = if text.starts_with(&quote.to_string().repeat(3)) {
        3
    } else {
        1
    };
    if !matches!(quote, '\'' | '"')
        || text.len() < width * 2
        || !text.ends_with(&quote.to_string().repeat(width))
    {
        return None;
    }
    // Validate against the TOML parser rather than guessing malformed escapes.
    let _: toml::Table = toml::from_str(&format!("value = {text}")).ok()?;
    let range = node.start_byte() + width..node.end_byte() - width;
    let mut cursor = range.start;
    if width == 3 {
        if source[cursor..range.end].starts_with("\r\n") {
            cursor += 2;
        } else if source[cursor..range.end].starts_with('\n') {
            cursor += 1;
        }
    }
    let mut units = Vec::new();
    while cursor < range.end {
        let start = cursor;
        let ch = source[cursor..range.end].chars().next()?;
        cursor += ch.len_utf8();
        if ch == '\r' && source[cursor..range.end].starts_with('\n') {
            cursor += 1;
            units.push(('\n', start..cursor));
        } else if quote == '"' && ch == '\\' {
            let marker = source[cursor..range.end].chars().next()?;
            if width == 3 && marker.is_ascii_whitespace() {
                while cursor < range.end && source.as_bytes()[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                continue;
            }
            let count = match marker {
                'u' => 5,
                'U' => 9,
                _ => 1,
            };
            let end = cursor.checked_add(count)?;
            let escape = source.get(start..end)?;
            let parsed: toml::Table = toml::from_str(&format!("value = \"{escape}\"")).ok()?;
            for ch in parsed.get("value")?.as_str()?.chars() {
                units.push((ch, start..end));
            }
            cursor = end;
        } else {
            units.push((ch, start..cursor));
        }
    }
    Some((range, units))
}

fn project_units(units: &[MappedScalar], decoded: &str, source: &str) -> Vec<Segment> {
    let mut result = Vec::new();
    let mut offset = 0;
    let mut skip_until = 0;
    for (ch, range) in units {
        if offset >= skip_until {
            let rest = &decoded[offset..];
            let template = [
                ("{{", "}}", "kat_mise_value"),
                ("{%", "%}", ""),
                ("{#", "#}", ""),
            ]
            .into_iter()
            .find(|(open, _, _)| rest.starts_with(open));
            if let Some((_, close, placeholder)) = template {
                skip_until = rest
                    .find(close)
                    .map_or(decoded.len(), |end| offset + end + close.len());
                let mut replacement = placeholder.to_owned();
                replacement.extend(decoded[offset..skip_until].chars().filter(|ch| *ch == '\n'));
                result.push(Segment::Synthetic(replacement));
            } else if source.get(range.clone()) == Some(ch.to_string().as_str()) {
                if let Some(Segment::Source(previous)) = result.last_mut()
                    && previous.end == range.start
                {
                    previous.end = range.end;
                } else {
                    result.push(Segment::Source(range.clone()));
                }
            } else {
                result.push(Segment::MappedText {
                    text: ch.to_string(),
                    source: range.clone(),
                });
            }
        }
        offset += ch.len_utf8();
    }
    result
}
