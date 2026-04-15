use std::{env, ffi::OsStr, io::IsTerminal};

use anstyle::RgbColor;
use serde::Serialize;
use termwiz::caps::{Capabilities, ColorLevel};

use crate::{
    debug_runtime_log,
    render_ops::RenderPlanSnapshot,
    terminal_background::detect_nested_region_tint,
    theme::{ColorMode, RgbColorSnapshot, Theme},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalCapabilities {
    color_mode: ColorMode,
    nested_region_tint: Option<RgbColor>,
    nested_region_tint_source: NestedRegionTintSource,
    background_queries_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NestedRegionTintSource {
    Probed,
    ProbeUnavailable,
    UnsupportedColorMode,
    QueriesDisabled,
    DebugDisabled,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        let color_mode = detect_color_mode();
        let background_queries_enabled =
            !background_queries_disabled(env::var_os("KAT_DISABLE_TERMINAL_QUERIES").as_deref());
        let (nested_region_tint, nested_region_tint_source) = if color_mode != ColorMode::TrueColor
        {
            (None, NestedRegionTintSource::UnsupportedColorMode)
        } else if background_queries_enabled {
            match detect_nested_region_tint(color_mode) {
                Some(tint) => (Some(tint), NestedRegionTintSource::Probed),
                None => (None, NestedRegionTintSource::ProbeUnavailable),
            }
        } else {
            (None, NestedRegionTintSource::QueriesDisabled)
        };

        let capabilities = Self {
            color_mode,
            nested_region_tint,
            nested_region_tint_source,
            background_queries_enabled,
        };
        debug_runtime_log::log(
            "terminal_capabilities_detected",
            &serde_json::json!({
                "stdout_is_terminal": std::io::stdout().is_terminal(),
                "term": env::var_os("TERM").map(|value| value.to_string_lossy().into_owned()),
                "colorterm": env::var_os("COLORTERM").map(|value| value.to_string_lossy().into_owned()),
                "term_program": env::var_os("TERM_PROGRAM").map(|value| value.to_string_lossy().into_owned()),
                "kat_color_mode": env::var_os("KAT_COLOR_MODE").map(|value| value.to_string_lossy().into_owned()),
                "capabilities": capabilities.snapshot(),
            }),
        );
        capabilities
    }

    pub fn for_debug_layers() -> Self {
        Self {
            color_mode: detect_color_mode(),
            nested_region_tint: None,
            nested_region_tint_source: NestedRegionTintSource::DebugDisabled,
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
            nested_region_tint_source: self.nested_region_tint_source,
            background_queries_enabled: self.background_queries_enabled,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TerminalCapabilitiesSnapshot {
    pub color_mode: ColorMode,
    pub nested_region_tint: Option<RgbColorSnapshot>,
    pub nested_region_tint_source: NestedRegionTintSource,
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
        let detected = match explicit_mode
            .to_string_lossy()
            .to_ascii_lowercase()
            .as_str()
        {
            "none" | "never" | "off" => ColorMode::NoColor,
            "ansi" | "16" => ColorMode::Ansi,
            "truecolor" | "24bit" | "rgb" => ColorMode::TrueColor,
            _ => ColorMode::Ansi,
        };
        debug_runtime_log::log(
            "terminal_color_mode_detected",
            &serde_json::json!({
                "source": "kat_color_mode",
                "value": explicit_mode.to_string_lossy().into_owned(),
                "detected": detected,
            }),
        );
        return detected;
    }

    if env::var_os("NO_COLOR").is_some() {
        debug_runtime_log::log(
            "terminal_color_mode_detected",
            &serde_json::json!({
                "source": "no_color",
                "detected": ColorMode::NoColor,
            }),
        );
        return ColorMode::NoColor;
    }

    if env::var("TERM").is_ok_and(|term| term == "dumb") {
        debug_runtime_log::log(
            "terminal_color_mode_detected",
            &serde_json::json!({
                "source": "term_dumb",
                "detected": ColorMode::NoColor,
            }),
        );
        return ColorMode::NoColor;
    }

    let detected = Capabilities::new_from_env()
        .map(|capabilities| match capabilities.color_level() {
            ColorLevel::TrueColor => ColorMode::TrueColor,
            ColorLevel::TwoFiftySix | ColorLevel::Sixteen => ColorMode::Ansi,
            ColorLevel::MonoChrome => ColorMode::NoColor,
        })
        .unwrap_or(ColorMode::Ansi);
    debug_runtime_log::log(
        "terminal_color_mode_detected",
        &serde_json::json!({
            "source": "termwiz_capabilities",
            "detected": detected,
        }),
    );
    detected
}

#[cfg(test)]
mod tests {
    use super::{
        NestedRegionTintSource, TerminalCapabilities, background_queries_disabled,
        escape_control_sequences,
    };
    use std::ffi::OsStr;

    #[test]
    fn debug_capabilities_disable_terminal_queries() {
        let capabilities = TerminalCapabilities::for_debug_layers();
        assert!(!capabilities.snapshot().background_queries_enabled);
        assert_eq!(
            capabilities.snapshot().nested_region_tint_source,
            NestedRegionTintSource::DebugDisabled
        );
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
