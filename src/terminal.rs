use std::{env, ffi::OsStr};

use anstyle::RgbColor;
use serde::Serialize;

use crate::{
    render_ops::RenderPlanSnapshot,
    terminal_background::detect_nested_region_tint,
    theme::{ColorMode, RgbColorSnapshot, Theme},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalCapabilities {
    color_mode: ColorMode,
    nested_region_tint: Option<RgbColor>,
    background_queries_enabled: bool,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        let color_mode = detect_color_mode();
        let background_queries_enabled =
            !background_queries_disabled(env::var_os("KAT_DISABLE_TERMINAL_QUERIES").as_deref());
        let nested_region_tint = if background_queries_enabled {
            detect_nested_region_tint(color_mode)
        } else {
            None
        };

        Self {
            color_mode,
            nested_region_tint,
            background_queries_enabled,
        }
    }

    pub fn for_debug_layers() -> Self {
        Self {
            color_mode: detect_color_mode(),
            nested_region_tint: None,
            background_queries_enabled: false,
        }
    }

    pub fn theme(self) -> Theme {
        Theme::new(self.color_mode, self.nested_region_tint)
    }

    pub fn snapshot(self) -> TerminalCapabilitiesSnapshot {
        TerminalCapabilitiesSnapshot {
            color_mode: self.color_mode,
            nested_region_tint: self.nested_region_tint.map(Into::into),
            background_queries_enabled: self.background_queries_enabled,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalCapabilitiesSnapshot {
    pub color_mode: ColorMode,
    pub nested_region_tint: Option<RgbColorSnapshot>,
    pub background_queries_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalRenderSnapshot {
    pub capabilities: TerminalCapabilitiesSnapshot,
    pub render_plan: RenderPlanSnapshot,
    pub encoded_output: String,
    pub encoded_bytes: usize,
}

pub fn escape_control_sequences(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\u{1b}' => escaped.push_str("\\e"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn background_queries_disabled(value: Option<&OsStr>) -> bool {
    match value.and_then(|value| value.to_str()) {
        None => false,
        Some(value) => {
            let value = value.trim().to_ascii_lowercase();
            matches!(value.as_str(), "" | "1" | "true" | "yes" | "on")
        }
    }
}

fn detect_color_mode() -> ColorMode {
    if let Some(explicit_mode) = env::var_os("KAT_COLOR_MODE") {
        return match explicit_mode
            .to_string_lossy()
            .to_ascii_lowercase()
            .as_str()
        {
            "none" | "never" | "off" => ColorMode::NoColor,
            "ansi" | "16" => ColorMode::Ansi,
            "truecolor" | "24bit" | "rgb" => ColorMode::TrueColor,
            _ => ColorMode::Ansi,
        };
    }

    if env::var_os("NO_COLOR").is_some() {
        return ColorMode::NoColor;
    }

    if env::var("TERM").is_ok_and(|term| term == "dumb") {
        return ColorMode::NoColor;
    }

    if env::var("COLORTERM").is_ok_and(|value| {
        let value = value.to_ascii_lowercase();
        value.contains("truecolor") || value.contains("24bit")
    }) {
        return ColorMode::TrueColor;
    }

    if env::var("TERM").is_ok_and(|term| {
        let term = term.to_ascii_lowercase();
        term.contains("direct")
            || term.contains("kitty")
            || term.contains("wezterm")
            || term.contains("ghostty")
    }) {
        return ColorMode::TrueColor;
    }

    if env::var("TERM_PROGRAM").is_ok_and(|program| {
        matches!(
            program.as_str(),
            "WezTerm" | "iTerm.app" | "vscode" | "WarpTerminal" | "ghostty"
        )
    }) {
        return ColorMode::TrueColor;
    }

    ColorMode::Ansi
}

#[cfg(test)]
mod tests {
    use super::{TerminalCapabilities, background_queries_disabled, escape_control_sequences};
    use std::ffi::OsStr;

    #[test]
    fn debug_capabilities_disable_terminal_queries() {
        let capabilities = TerminalCapabilities::for_debug_layers();
        assert!(!capabilities.snapshot().background_queries_enabled);
    }

    #[test]
    fn env_values_disable_terminal_queries() {
        for value in ["", "1", "true", "TRUE", "yes", "on"] {
            assert!(background_queries_disabled(Some(OsStr::new(value))));
        }
    }

    #[test]
    fn escaped_control_sequences_are_visible() {
        assert_eq!(
            escape_control_sequences("\x1b[31mhi\r\n"),
            "\\e[31mhi\\r\\n"
        );
    }
}
