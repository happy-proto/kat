use std::time::Duration;

use anstyle::RgbColor;
use terminal_colorsaurus::{QueryOptions, background_color};

use crate::theme::ColorMode;

pub(crate) fn detect_nested_region_tint(color_mode: ColorMode) -> Option<RgbColor> {
    if color_mode != ColorMode::TrueColor {
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
    let target = mix_rgb(background, RgbColor(68, 71, 90), 0.65);
    Some(mix_rgb(background, target, 0.36))
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
