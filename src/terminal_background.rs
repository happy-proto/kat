use std::ffi::OsStr;
use std::time::Duration;

use anstyle::RgbColor;
use terminal_colorsaurus::{QueryOptions, background_color};

use crate::theme::ColorMode;

pub(crate) fn detect_nested_region_tint(color_mode: ColorMode) -> Option<RgbColor> {
    if color_mode != ColorMode::TrueColor {
        return None;
    }
    if terminal_background_queries_disabled(env_var("KAT_DISABLE_TERMINAL_QUERIES").as_deref()) {
        return None;
    }

    let mut options = QueryOptions::default();
    options.timeout = Duration::from_millis(120);

    // `terminal-colorsaurus` 这里只是当前阶段的 OSC 11 查询后端。
    // 我们把它限制在这个很窄的接口后面，避免 renderer/theme 直接绑定第二套
    // terminal I/O 栈；否则未来一旦引入更完整的 terminal API 抽象，很容易在
    // TTY 所有权、raw mode 和查询时序上发生冲突。长期应迁移到 kat 自己的
    // terminal 层，理想目标是基于支持颜色查询的 termwiz 维护分支来统一承接。
    let background = background_color(options).ok()?;
    let (r, g, b) = background.scale_to_8bit();
    let background = RgbColor(r, g, b);
    let lifted = mix_rgb(background, RgbColor(68, 71, 90), 0.55);
    Some(mix_rgb(background, lifted, 0.5))
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
    use super::terminal_background_queries_disabled;
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
}
