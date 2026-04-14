use std::{ffi::OsStr, io::IsTerminal};

use anstyle::RgbColor;
use termwiz::{
    caps::Capabilities,
    color::SrgbaTuple,
    escape::osc::DynamicColorNumber,
    terminal::{Terminal, new_terminal},
};

use crate::{debug_runtime_log, theme::ColorMode};

pub(crate) fn detect_nested_region_tint(color_mode: ColorMode) -> Option<RgbColor> {
    if color_mode != ColorMode::TrueColor {
        debug_runtime_log::log(
            "nested_region_tint_probe_skipped",
            &serde_json::json!({
                "reason": "non_truecolor_mode",
                "color_mode": color_mode,
            }),
        );
        return None;
    }
    if terminal_background_queries_disabled(env_var("KAT_DISABLE_TERMINAL_QUERIES").as_deref()) {
        debug_runtime_log::log(
            "nested_region_tint_probe_skipped",
            &serde_json::json!({
                "reason": "queries_disabled",
            }),
        );
        return None;
    }
    if !std::io::stdout().is_terminal() {
        debug_runtime_log::log(
            "nested_region_tint_probe_skipped",
            &serde_json::json!({
                "reason": "stdout_not_tty",
            }),
        );
        return None;
    }

    let caps = match Capabilities::new_from_env() {
        Ok(caps) => caps,
        Err(error) => {
            debug_runtime_log::log(
                "nested_region_tint_probe_failed",
                &serde_json::json!({
                    "stage": "capabilities_from_env",
                    "error": error.to_string(),
                }),
            );
            return None;
        }
    };
    let color_level = caps.color_level();
    debug_runtime_log::log(
        "nested_region_tint_probe_started",
        &serde_json::json!({
            "term": std::env::var_os("TERM").map(|value| value.to_string_lossy().into_owned()),
            "colorterm": std::env::var_os("COLORTERM").map(|value| value.to_string_lossy().into_owned()),
            "term_program": std::env::var_os("TERM_PROGRAM").map(|value| value.to_string_lossy().into_owned()),
            "color_level": format!("{color_level:?}"),
        }),
    );

    let mut terminal = match new_terminal(caps) {
        Ok(terminal) => terminal,
        Err(error) => {
            debug_runtime_log::log(
                "nested_region_tint_probe_failed",
                &serde_json::json!({
                    "stage": "new_terminal",
                    "error": error.to_string(),
                }),
            );
            return None;
        }
    };
    if let Err(error) = terminal.set_raw_mode() {
        debug_runtime_log::log(
            "nested_region_tint_probe_failed",
            &serde_json::json!({
                "stage": "set_raw_mode",
                "error": error.to_string(),
            }),
        );
        return None;
    }
    debug_runtime_log::log(
        "nested_region_tint_probe_raw_mode_enabled",
        &serde_json::json!({}),
    );
    let mut probe = match terminal.probe_capabilities() {
        Some(probe) => probe,
        None => {
            debug_runtime_log::log(
                "nested_region_tint_probe_failed",
                &serde_json::json!({
                    "stage": "probe_capabilities",
                    "error": "probe_unavailable",
                }),
            );
            return None;
        }
    };

    let foreground = probe
        .dynamic_color(DynamicColorNumber::TextForegroundColor)
        .map_err(|error| error.to_string())
        .map(srgba_to_rgb);
    let background = probe
        .dynamic_color(DynamicColorNumber::TextBackgroundColor)
        .map_err(|error| error.to_string())
        .map(srgba_to_rgb);

    debug_runtime_log::log(
        "nested_region_tint_probe_colors",
        &serde_json::json!({
            "foreground": foreground
                .as_ref()
                .map(|color| format_rgb(*color))
                .unwrap_or_else(|error| format!("error:{error}")),
            "background": background
                .as_ref()
                .map(|color| format_rgb(*color))
                .unwrap_or_else(|error| format!("error:{error}")),
        }),
    );

    let foreground = foreground.ok()?;
    let background = background.ok()?;

    let tint = derive_nested_region_tint(foreground, background);
    debug_runtime_log::log(
        "nested_region_tint_probe_succeeded",
        &serde_json::json!({
            "foreground": format_rgb(foreground),
            "background": format_rgb(background),
            "tint": format_rgb(tint),
        }),
    );
    Some(tint)
}

fn derive_nested_region_tint(foreground: RgbColor, background: RgbColor) -> RgbColor {
    let contrast_lift = mix_rgb(background, foreground, 0.08);
    let palette_lift = mix_rgb(contrast_lift, RgbColor(68, 71, 90), 0.55);
    mix_rgb(contrast_lift, palette_lift, 0.5)
}

fn srgba_to_rgb(color: SrgbaTuple) -> RgbColor {
    let (r, g, b, _) = color.to_srgb_u8();
    RgbColor(r, g, b)
}

fn format_rgb(color: RgbColor) -> String {
    format!("rgb({},{},{})", color.0, color.1, color.2)
}

fn env_var(name: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(name)
}

fn terminal_background_queries_disabled(value: Option<&OsStr>) -> bool {
    match value.and_then(|value| value.to_str()) {
        None => false,
        Some(value) => {
            let value = value.trim().to_ascii_lowercase();
            matches!(value.as_str(), "" | "1" | "true" | "yes" | "on")
        }
    }
}

fn mix_rgb(base: RgbColor, other: RgbColor, weight: f32) -> RgbColor {
    fn mix_channel(base: u8, other: u8, weight: f32) -> u8 {
        let base = base as f32;
        let other = other as f32;
        (base + (other - base) * weight).round().clamp(0.0, 255.0) as u8
    }

    RgbColor(
        mix_channel(base.0, other.0, weight),
        mix_channel(base.1, other.1, weight),
        mix_channel(base.2, other.2, weight),
    )
}

#[cfg(test)]
mod tests {
    use super::{derive_nested_region_tint, terminal_background_queries_disabled};
    use anstyle::RgbColor;
    use std::ffi::OsStr;

    #[test]
    fn terminal_queries_are_enabled_by_default() {
        assert!(!terminal_background_queries_disabled(None));
    }

    #[test]
    fn terminal_queries_can_be_disabled_via_env_var() {
        for value in ["", "1", "true", "TRUE", "yes", "on"] {
            assert!(
                terminal_background_queries_disabled(Some(OsStr::new(value))),
                "expected {value:?} to disable terminal background queries"
            );
        }
    }

    #[test]
    fn non_disable_values_keep_terminal_queries_enabled() {
        for value in ["0", "false", "no", "off", "kat"] {
            assert!(
                !terminal_background_queries_disabled(Some(OsStr::new(value))),
                "expected {value:?} to keep terminal background queries enabled"
            );
        }
    }

    #[test]
    fn tint_uses_foreground_and_background() {
        let tint = derive_nested_region_tint(RgbColor(248, 248, 242), RgbColor(40, 42, 54));
        assert_ne!(tint, RgbColor(40, 42, 54));
    }
}
