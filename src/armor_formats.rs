use std::ops::Range;

use anyhow::Result;

use crate::{
    HighlightRenderData, StyledSpan,
    document_kind::DocumentKind,
    theme::{Theme, TokenStyle},
};

const PEM_LABELS: &[&str] = &[
    "CERTIFICATE",
    "TRUSTED CERTIFICATE",
    "CERTIFICATE REQUEST",
    "NEW CERTIFICATE REQUEST",
    "X509 CRL",
    "PUBLIC KEY",
    "PRIVATE KEY",
    "ENCRYPTED PRIVATE KEY",
    "RSA PUBLIC KEY",
    "RSA PRIVATE KEY",
    "DSA PRIVATE KEY",
    "EC PRIVATE KEY",
    "EC PARAMETERS",
    "DH PARAMETERS",
    "OPENSSH PRIVATE KEY",
    "PKCS7",
    "CMS",
];

const OPENPGP_LABELS: &[&str] = &[
    "PGP MESSAGE",
    "PGP PUBLIC KEY BLOCK",
    "PGP PRIVATE KEY BLOCK",
    "PGP SIGNATURE",
    "PGP SIGNED MESSAGE",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArmorKind {
    Pem,
    OpenPgp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundaryKind {
    Begin,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Boundary<'a> {
    kind: BoundaryKind,
    label: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpenPgpState {
    Armor,
    SignedHeaders,
    SignedText,
}

pub(crate) fn render_armor_format(
    document_kind: DocumentKind,
    source: &str,
    theme: &Theme,
) -> Result<HighlightRenderData> {
    let kind = match document_kind.runtime_name() {
        "pem" => ArmorKind::Pem,
        "openpgp_armor" => ArmorKind::OpenPgp,
        runtime_name => anyhow::bail!("unsupported armor runtime {runtime_name}"),
    };

    Ok(HighlightRenderData {
        resolved_document_kind: document_kind,
        spans: armor_spans(kind, source, theme),
        nested_regions: Vec::new(),
    })
}

pub(crate) fn looks_like_pem(source: &str) -> bool {
    looks_like_armor(source, ArmorKind::Pem)
}

pub(crate) fn looks_like_openpgp_armor(source: &str) -> bool {
    looks_like_armor(source, ArmorKind::OpenPgp)
}

pub(crate) fn is_pem_extension(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "pem" | "crt" | "cer" | "csr" | "crl" | "key"
    )
}

pub(crate) fn is_openpgp_armor_extension(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "asc" | "pgp" | "gpg"
    )
}

fn looks_like_armor(source: &str, kind: ArmorKind) -> bool {
    let mut saw_begin = None;

    for line in source.lines() {
        let trimmed = line.trim();
        let Some(boundary) = parse_boundary(trimmed, kind) else {
            continue;
        };

        match boundary.kind {
            BoundaryKind::Begin => saw_begin = Some(boundary.label),
            BoundaryKind::End if saw_begin == Some(boundary.label) => return true,
            BoundaryKind::End => {}
        }
    }

    false
}

fn armor_spans(kind: ArmorKind, source: &str, theme: &Theme) -> Vec<StyledSpan> {
    let mut spans = Vec::new();
    let mut offset = 0usize;
    let mut pgp_state = OpenPgpState::Armor;

    for line in source.split_inclusive('\n') {
        let line_body = line.strip_suffix('\n').unwrap_or(line);
        let line_body = line_body.strip_suffix('\r').unwrap_or(line_body);
        style_line(
            kind,
            source,
            line_body,
            offset,
            theme,
            &mut spans,
            &mut pgp_state,
        );
        offset += line.len();
    }

    if offset < source.len() {
        style_line(
            kind,
            source,
            &source[offset..],
            offset,
            theme,
            &mut spans,
            &mut pgp_state,
        );
    }

    if spans.is_empty() && !source.is_empty() {
        push_span(&mut spans, 0..source.len(), theme.default_style());
    }

    spans
}

fn style_line(
    kind: ArmorKind,
    source: &str,
    line: &str,
    line_start: usize,
    theme: &Theme,
    spans: &mut Vec<StyledSpan>,
    pgp_state: &mut OpenPgpState,
) {
    let line_end = line_start + line.len();
    let trimmed_start = line.len() - line.trim_start_matches([' ', '\t']).len();
    let trimmed = &line[trimmed_start..];

    if let Some(boundary) = parse_boundary(trimmed, kind) {
        style_boundary_line(
            source,
            trimmed,
            line_start + trimmed_start,
            boundary,
            theme,
            spans,
        );
        if kind == ArmorKind::OpenPgp {
            *pgp_state =
                if boundary.kind == BoundaryKind::Begin && boundary.label == "PGP SIGNED MESSAGE" {
                    OpenPgpState::SignedHeaders
                } else {
                    OpenPgpState::Armor
                };
        }
        return;
    }

    if line.trim().is_empty() {
        if kind == ArmorKind::OpenPgp && *pgp_state == OpenPgpState::SignedHeaders {
            *pgp_state = OpenPgpState::SignedText;
        }
        return;
    }

    if kind == ArmorKind::OpenPgp && *pgp_state == OpenPgpState::SignedText {
        return;
    }

    if let Some((key, separator, value)) = split_header_like_line(line) {
        push_styled_text(
            spans,
            absolute_range(line_start, key),
            "property",
            source,
            theme,
        );
        push_styled_text(
            spans,
            absolute_range(line_start, separator),
            "punctuation.delimiter",
            source,
            theme,
        );
        push_styled_text(
            spans,
            absolute_range(line_start, value),
            "string",
            source,
            theme,
        );
        return;
    }

    if kind == ArmorKind::OpenPgp && is_openpgp_checksum(trimmed) {
        let start = line_start + trimmed_start;
        push_styled_text(spans, start..line_end, "string.special", source, theme);
        return;
    }

    if is_base64_line(trimmed) {
        let start = line_start + trimmed_start;
        push_styled_text(spans, start..line_end, "constant", source, theme);
    }
}

fn style_boundary_line(
    source: &str,
    line: &str,
    line_start: usize,
    boundary: Boundary<'_>,
    theme: &Theme,
    spans: &mut Vec<StyledSpan>,
) {
    let prefix = match boundary.kind {
        BoundaryKind::Begin => "-----BEGIN ",
        BoundaryKind::End => "-----END ",
    };
    let suffix = "-----";
    let label_start = prefix.len();
    let label_end = line.len().saturating_sub(suffix.len());
    let label_capture = if is_private_or_secret_label(boundary.label) {
        "text.warning"
    } else {
        "type"
    };

    push_styled_text(
        spans,
        line_start..line_start + prefix.len(),
        "keyword.directive",
        source,
        theme,
    );
    push_styled_text(
        spans,
        line_start + label_start..line_start + label_end,
        label_capture,
        source,
        theme,
    );
    push_styled_text(
        spans,
        line_start + label_end..line_start + line.len(),
        "keyword.directive",
        source,
        theme,
    );
}

fn absolute_range(offset: usize, range: Range<usize>) -> Range<usize> {
    offset + range.start..offset + range.end
}

fn parse_boundary(line: &str, kind: ArmorKind) -> Option<Boundary<'_>> {
    let (boundary_kind, rest) = line
        .strip_prefix("-----BEGIN ")
        .map(|rest| (BoundaryKind::Begin, rest))
        .or_else(|| {
            line.strip_prefix("-----END ")
                .map(|rest| (BoundaryKind::End, rest))
        })?;
    let label = rest.strip_suffix("-----")?;

    is_known_label(kind, label).then_some(Boundary {
        kind: boundary_kind,
        label,
    })
}

fn is_known_label(kind: ArmorKind, label: &str) -> bool {
    match kind {
        ArmorKind::Pem => PEM_LABELS.contains(&label),
        ArmorKind::OpenPgp => OPENPGP_LABELS.contains(&label),
    }
}

fn is_private_or_secret_label(label: &str) -> bool {
    label.contains("PRIVATE") || label.contains("SECRET")
}

fn split_header_like_line(line: &str) -> Option<(Range<usize>, Range<usize>, Range<usize>)> {
    let trimmed_start = line.len() - line.trim_start_matches([' ', '\t']).len();
    let trimmed = &line[trimmed_start..];

    let separator_index = trimmed.find(':').or_else(|| {
        (trimmed.starts_with("subject=") || trimmed.starts_with("issuer=")).then_some(7)
    })?;
    let key_end = trimmed_start + separator_index;
    let separator_start = key_end;
    let separator_end = separator_start + 1;
    let value_start = separator_end;

    Some((
        trimmed_start..key_end,
        separator_start..separator_end,
        value_start..line.len(),
    ))
}

fn is_openpgp_checksum(line: &str) -> bool {
    let Some(checksum) = line.strip_prefix('=') else {
        return false;
    };

    checksum.len() == 4 && checksum.bytes().all(is_base64_byte)
}

fn is_base64_line(line: &str) -> bool {
    !line.is_empty()
        && line.len() >= 4
        && line
            .bytes()
            .all(|byte| is_base64_byte(byte) || byte == b'=')
}

fn is_base64_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/')
}

fn push_styled_text(
    spans: &mut Vec<StyledSpan>,
    range: Range<usize>,
    capture: &str,
    source: &str,
    theme: &Theme,
) {
    if range.start >= range.end {
        return;
    }

    let style = theme.token_style_for(capture, &source[range.clone()]);
    push_span(spans, range, style);
}

fn push_span(spans: &mut Vec<StyledSpan>, range: Range<usize>, style: Option<TokenStyle>) {
    if range.start >= range.end {
        return;
    }

    if let Some(last) = spans.last_mut()
        && last.range.end == range.start
        && last.style == style
        && last.hyperlink.is_none()
    {
        last.range.end = range.end;
        return;
    }

    spans.push(StyledSpan {
        range,
        style,
        hyperlink: None,
    });
}

#[cfg(test)]
mod tests {
    use crate::{
        armor_formats::{looks_like_openpgp_armor, looks_like_pem},
        theme::{ColorMode, Theme},
    };

    use super::{ArmorKind, armor_spans};

    #[test]
    fn detects_matching_pem_boundaries() {
        assert!(looks_like_pem(
            "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n"
        ));
        assert!(!looks_like_pem(
            "-----BEGIN CERTIFICATE-----\nMIIB\n-----END PUBLIC KEY-----\n"
        ));
    }

    #[test]
    fn detects_matching_openpgp_boundaries() {
        assert!(looks_like_openpgp_armor(
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nmQEN\n=abcd\n-----END PGP PUBLIC KEY BLOCK-----\n"
        ));
        assert!(!looks_like_openpgp_armor(
            "-----BEGIN PGP MESSAGE-----\n\nmQEN\n-----END PGP SIGNATURE-----\n"
        ));
    }

    #[test]
    fn keeps_cleartext_signed_message_body_unstyled() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let source = "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA256\n\nhello: world\n-----BEGIN PGP SIGNATURE-----\n\nabcD\n=1234\n-----END PGP SIGNATURE-----\n";
        let spans = armor_spans(ArmorKind::OpenPgp, source, &theme);
        let body_start = source.find("hello").expect("body should exist");

        assert!(
            !spans
                .iter()
                .any(|span| span.range.start <= body_start && body_start < span.range.end),
            "cleartext signed body should remain plain text"
        );
    }
}
