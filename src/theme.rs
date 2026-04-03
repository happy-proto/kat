use std::env;

use anstyle::{AnsiColor, Color, RgbColor, Style};

use crate::terminal_background::detect_nested_region_tint;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorMode {
    NoColor,
    Ansi,
    TrueColor,
}

pub struct Theme {
    color_mode: ColorMode,
    nested_region_tint: Option<RgbColor>,
}

impl Theme {
    pub fn detect() -> Self {
        let color_mode = detect_color_mode();
        Self {
            nested_region_tint: detect_nested_region_tint(color_mode),
            color_mode,
        }
    }

    #[cfg(test)]
    pub fn for_mode(color_mode: ColorMode) -> Self {
        Self {
            color_mode,
            nested_region_tint: None,
        }
    }

    #[cfg(test)]
    pub fn for_mode_with_nested_region_tint(
        color_mode: ColorMode,
        nested_region_tint: Option<RgbColor>,
    ) -> Self {
        Self {
            color_mode,
            nested_region_tint,
        }
    }

    #[cfg(test)]
    pub(crate) fn nested_region_background(&self, level: usize) -> Option<RgbColor> {
        self.nested_region_tint
            .map(|background| adjust_nested_region_tint(background, level))
    }

    pub(crate) fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    pub(crate) fn default_style(&self) -> Option<TokenStyle> {
        if self.color_mode == ColorMode::NoColor {
            return None;
        }

        Some(TokenStyle::new(DraculaColor::Foreground).with_color_priority(0))
    }

    #[cfg(test)]
    pub fn style_for(&self, capture: &str, text: &str) -> Option<Style> {
        self.token_style_for(capture, text)
            .map(|token| token.to_style(self.color_mode))
    }

    pub(crate) fn token_style_for(&self, capture: &str, text: &str) -> Option<TokenStyle> {
        if self.color_mode == ColorMode::NoColor {
            return None;
        }

        Some(if matches_instance_reserved_word(capture, text) {
            TokenStyle::instance_reserved_word()
        } else {
            token_style_for(capture, text)
        })
    }

    pub(crate) fn merged_token_style_for<'a>(
        &self,
        captures: impl IntoIterator<Item = &'a str>,
        text: &str,
    ) -> Option<TokenStyle> {
        if self.color_mode == ColorMode::NoColor {
            return None;
        }

        captures.into_iter().fold(None, |merged, capture| {
            let token = self.token_style_for(capture, text);
            match (merged, token) {
                (Some(current), Some(next)) => Some(current.merge(next)),
                (None, Some(next)) => Some(next),
                (Some(current), None) => Some(current),
                (None, None) => None,
            }
        })
    }

    pub(crate) fn nested_region_tint(&self, level: usize) -> Option<TokenStyle> {
        if level == 0 {
            return None;
        }

        self.nested_region_tint.map(|background| {
            TokenStyle::background_tint(adjust_nested_region_tint(background, level))
        })
    }
}

fn adjust_nested_region_tint(background: RgbColor, level: usize) -> RgbColor {
    let lift_target = RgbColor(98, 114, 164);
    let lift = nested_region_lift_weight(level);
    mix_rgb(background, lift_target, lift)
}

fn nested_region_lift_weight(level: usize) -> f32 {
    match level {
        0 => 0.0,
        1 => 0.2,
        2 => 0.34,
        3 => 0.46,
        4 => 0.58,
        5 => 0.68,
        6 => 0.76,
        _ => 0.82,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TokenStyle {
    color: DraculaColor,
    color_priority: u8,
    background: Option<RgbColor>,
    italic: bool,
    bold: bool,
    underline: bool,
    strikethrough: bool,
}

impl TokenStyle {
    const fn new(color: DraculaColor) -> Self {
        Self {
            color,
            color_priority: color.priority(),
            background: None,
            italic: false,
            bold: false,
            underline: false,
            strikethrough: false,
        }
    }

    const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    const fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    const fn instance_reserved_word() -> Self {
        Self::new(DraculaColor::Purple).italic()
    }

    const fn with_color_priority(mut self, color_priority: u8) -> Self {
        self.color_priority = color_priority;
        self
    }

    fn background_tint(background: RgbColor) -> Self {
        Self {
            background: Some(background),
            ..Self::new(DraculaColor::Foreground).with_color_priority(0)
        }
    }

    pub(crate) fn merge(self, overlay: Self) -> Self {
        let (color, color_priority) = if overlay.color_priority >= self.color_priority {
            (overlay.color, overlay.color_priority)
        } else {
            (self.color, self.color_priority)
        };

        Self {
            color,
            color_priority,
            background: overlay.background.or(self.background),
            italic: (self.italic && self.color_priority >= color_priority)
                || (overlay.italic && overlay.color_priority >= color_priority),
            bold: (self.bold && self.color_priority >= color_priority)
                || (overlay.bold && overlay.color_priority >= color_priority),
            underline: (self.underline && self.color_priority >= color_priority)
                || (overlay.underline && overlay.color_priority >= color_priority),
            strikethrough: (self.strikethrough && self.color_priority >= color_priority)
                || (overlay.strikethrough && overlay.color_priority >= color_priority),
        }
    }

    pub(crate) fn with_background_under(self, background: Self) -> Self {
        let mut merged = self;
        if merged.background.is_none() {
            merged.background = background.background;
        }
        merged
    }

    pub(crate) fn to_style(self, color_mode: ColorMode) -> Style {
        let mut style = Style::new().fg_color(Some(self.color.to_color(color_mode)));

        if let Some(background) = self.background {
            style = style.bg_color(Some(Color::Rgb(background)));
        }

        if self.italic {
            style = style.italic();
        }

        if self.bold {
            style = style.bold();
        }

        if self.underline {
            style = style.underline();
        }

        if self.strikethrough {
            style = style.strikethrough();
        }

        style
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DraculaColor {
    Comment,
    Foreground,
    Red,
    Orange,
    Yellow,
    Green,
    Purple,
    Cyan,
    Pink,
}

impl DraculaColor {
    const fn priority(self) -> u8 {
        match self {
            Self::Foreground | Self::Comment => 0,
            Self::Red
            | Self::Orange
            | Self::Yellow
            | Self::Green
            | Self::Purple
            | Self::Cyan
            | Self::Pink => 1,
        }
    }

    fn to_color(self, color_mode: ColorMode) -> Color {
        match color_mode {
            ColorMode::NoColor => unreachable!("NoColor never requests a concrete style"),
            ColorMode::Ansi => Color::Ansi(self.to_ansi_color()),
            ColorMode::TrueColor => Color::Rgb(self.to_rgb_color()),
        }
    }

    fn to_ansi_color(self) -> AnsiColor {
        match self {
            Self::Comment => AnsiColor::BrightBlack,
            Self::Foreground => AnsiColor::White,
            Self::Red => AnsiColor::Red,
            Self::Orange => AnsiColor::BrightYellow,
            Self::Yellow => AnsiColor::Yellow,
            Self::Green => AnsiColor::Green,
            Self::Purple => AnsiColor::Blue,
            Self::Cyan => AnsiColor::Cyan,
            Self::Pink => AnsiColor::Magenta,
        }
    }

    fn to_rgb_color(self) -> RgbColor {
        match self {
            Self::Comment => RgbColor(98, 114, 164),
            Self::Foreground => RgbColor(248, 248, 242),
            Self::Red => RgbColor(255, 85, 85),
            Self::Orange => RgbColor(255, 184, 108),
            Self::Yellow => RgbColor(241, 250, 140),
            Self::Green => RgbColor(80, 250, 123),
            Self::Purple => RgbColor(189, 147, 249),
            Self::Cyan => RgbColor(139, 233, 253),
            Self::Pink => RgbColor(255, 121, 198),
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

fn token_style_for(capture: &str, text: &str) -> TokenStyle {
    if capture == "keyword" && text == "new" {
        return TokenStyle::new(DraculaColor::Pink).bold();
    }

    match capture {
        "none" => TokenStyle::new(DraculaColor::Foreground).with_color_priority(0),
        "attribute" | "attribute.builtin" | "attribute.jsx" | "decorator" | "keyword.directive" => {
            TokenStyle::new(DraculaColor::Green).italic()
        }
        "boolean" | "float" | "number" => TokenStyle::new(DraculaColor::Orange),
        "comment" | "comment.doc" | "comment.documentation" => {
            TokenStyle::new(DraculaColor::Comment).italic()
        }
        "conditional"
        | "include"
        | "keyword"
        | "keyword.control"
        | "keyword.declaration"
        | "keyword.definition"
        | "keyword.import"
        | "keyword.jsdoc"
        | "keyword.operator"
        | "keyword.operator.regex" => TokenStyle::new(DraculaColor::Pink),
        "constant" | "constant.builtin" => TokenStyle::new(DraculaColor::Purple),
        "field" => TokenStyle::new(DraculaColor::Foreground).with_color_priority(0),
        "constructor"
        | "label.regex"
        | "property.json_key"
        | "property.toml"
        | "property.userscript"
        | "property.yaml"
        | "selector.class"
        | "selector.id"
        | "selector.pseudo"
        | "type"
        | "type.jsdoc"
        | "type.interface" => TokenStyle::new(DraculaColor::Cyan),
        "datetime" => TokenStyle::new(DraculaColor::Orange),
        "embedded" => TokenStyle::new(DraculaColor::Foreground).with_color_priority(0),
        "escape" | "operator" | "string.escape" | "operator.regex" | "string.escape.regex" => {
            TokenStyle::new(DraculaColor::Pink)
        }
        "punctuation.bracket.regex" => {
            TokenStyle::new(DraculaColor::Foreground).with_color_priority(2)
        }
        "punctuation.delimiter.regex" | "punctuation.special" => {
            TokenStyle::new(DraculaColor::Pink).with_color_priority(2)
        }
        "punctuation.delimiter.markdown" => {
            TokenStyle::new(DraculaColor::Foreground).with_color_priority(2)
        }
        "punctuation.list.markdown" => TokenStyle::new(DraculaColor::Cyan).with_color_priority(2),
        "punctuation.rule.markdown" => {
            TokenStyle::new(DraculaColor::Comment).with_color_priority(2)
        }
        "function"
        | "function.call"
        | "function.definition"
        | "function.macro"
        | "function.method"
        | "function.special"
        | "function.special.definition" => TokenStyle::new(DraculaColor::Green),
        "function.builtin" => TokenStyle::new(DraculaColor::Cyan),
        "tag" | "tag.jsx" => TokenStyle::new(DraculaColor::Pink),
        "tag.error" => TokenStyle::new(DraculaColor::Red).bold(),
        "invalid.illegal.regex" => TokenStyle::new(DraculaColor::Red)
            .underline()
            .with_color_priority(3),
        "identifier" | "storageclass" | "text" | "variable" | "property" => {
            TokenStyle::new(DraculaColor::Foreground).with_color_priority(0)
        }
        "parameter" | "variable.parameter" | "lifetime" => {
            TokenStyle::new(DraculaColor::Orange).italic()
        }
        "variable.builtin" | "variable.special" => TokenStyle::new(DraculaColor::Purple).italic(),
        "module" | "namespace" => TokenStyle::new(DraculaColor::Cyan).italic(),
        "punctuation.bracket" => TokenStyle::new(DraculaColor::Foreground).with_color_priority(2),
        "punctuation.delimiter" => TokenStyle::new(DraculaColor::Pink).with_color_priority(2),
        "number.quantifier.regex" => TokenStyle::new(DraculaColor::Orange),
        "regex" | "string.regex" => TokenStyle::new(DraculaColor::Red),
        "string" | "string.special" => TokenStyle::new(DraculaColor::Yellow),
        "string.special.key" => TokenStyle::new(DraculaColor::Cyan),
        "text.emphasis" => TokenStyle::new(DraculaColor::Yellow).italic(),
        "text.literal" => TokenStyle::new(DraculaColor::Yellow),
        "text.literal.block" => TokenStyle::new(DraculaColor::Orange),
        "text.literal.inline" => TokenStyle::new(DraculaColor::Green),
        "text.quote" => TokenStyle::new(DraculaColor::Yellow).italic(),
        "text.reference" => TokenStyle::new(DraculaColor::Pink),
        "text.strikethrough" => TokenStyle::new(DraculaColor::Comment).strikethrough(),
        "text.strong" => TokenStyle::new(DraculaColor::Orange).bold(),
        "text.title" => TokenStyle::new(DraculaColor::Purple).bold(),
        "text.uri" => TokenStyle::new(DraculaColor::Cyan),
        "type.qualifier" => TokenStyle::new(DraculaColor::Pink),
        "type.builtin" => TokenStyle::new(DraculaColor::Cyan).italic(),
        "type.unit" => TokenStyle::new(DraculaColor::Purple),
        "variable.jsdoc" => TokenStyle::new(DraculaColor::Foreground),
        "yaml.alias" => TokenStyle::new(DraculaColor::Green).italic().underline(),
        _ => token_style_for_prefix(capture),
    }
}

fn token_style_for_prefix(capture: &str) -> TokenStyle {
    match capture
        .split_once('.')
        .map_or(capture, |(prefix, _)| prefix)
    {
        "comment" => TokenStyle::new(DraculaColor::Comment).italic(),
        "constant" => TokenStyle::new(DraculaColor::Purple),
        "function" => TokenStyle::new(DraculaColor::Green),
        "keyword" | "conditional" | "include" | "operator" => TokenStyle::new(DraculaColor::Pink),
        "module" | "namespace" | "support" => TokenStyle::new(DraculaColor::Cyan).italic(),
        "number" => TokenStyle::new(DraculaColor::Orange),
        "property" | "variable" | "identifier" | "text" => {
            TokenStyle::new(DraculaColor::Foreground)
        }
        "selector" => TokenStyle::new(DraculaColor::Cyan),
        "tag" => TokenStyle::new(DraculaColor::Pink),
        "punctuation" => TokenStyle::new(DraculaColor::Foreground),
        "regex" => TokenStyle::new(DraculaColor::Red),
        "string" => TokenStyle::new(DraculaColor::Yellow),
        "type" => TokenStyle::new(DraculaColor::Cyan),
        _ => TokenStyle::new(DraculaColor::Foreground),
    }
}

fn matches_instance_reserved_word(capture: &str, text: &str) -> bool {
    capture != "variable.parameter" && matches!(text, "self" | "cls" | "this" | "super")
}

#[cfg(test)]
mod tests {
    use super::{ColorMode, Theme};

    #[test]
    fn maps_numbers_to_orange_in_truecolor_mode() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme.style_for("number", "42").expect("style should exist");

        assert_eq!(style.render().to_string(), "\u{1b}[38;2;255;184;108m");
    }

    #[test]
    fn maps_strings_to_ansi_yellow_in_ansi_mode() {
        let theme = Theme::for_mode(ColorMode::Ansi);
        let style = theme
            .style_for("string", "\"dracula\"")
            .expect("style should exist");

        assert_eq!(style.render().to_string(), "\u{1b}[33m");
    }

    #[test]
    fn highlights_instance_words_in_purple_italic() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("variable", "self")
            .expect("style should exist");

        assert_eq!(
            style.render().to_string(),
            "\u{1b}[3m\u{1b}[38;2;189;147;249m"
        );
    }

    #[test]
    fn keeps_properties_in_foreground() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("property", "package")
            .expect("style should exist");

        assert_eq!(style.render().to_string(), "\u{1b}[38;2;248;248;242m");
    }

    #[test]
    fn markdown_emphasis_uses_yellow_italic() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.emphasis", "italic")
            .expect("style should exist");

        assert_eq!(
            style.render().to_string(),
            "\u{1b}[3m\u{1b}[38;2;241;250;140m"
        );
    }

    #[test]
    fn markdown_strong_uses_orange_bold() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.strong", "strong")
            .expect("style should exist");

        assert_eq!(
            style.render().to_string(),
            "\u{1b}[1m\u{1b}[38;2;255;184;108m"
        );
    }

    #[test]
    fn markdown_heading_uses_purple_bold() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.title", "Heading")
            .expect("style should exist");

        assert_eq!(
            style.render().to_string(),
            "\u{1b}[1m\u{1b}[38;2;189;147;249m"
        );
    }

    #[test]
    fn markdown_inline_code_uses_green() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.literal.inline", "code")
            .expect("style should exist");

        assert_eq!(style.render().to_string(), "\u{1b}[38;2;80;250;123m");
    }

    #[test]
    fn markdown_link_uri_uses_cyan() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.uri", "https://example.com")
            .expect("style should exist");

        assert_eq!(style.render().to_string(), "\u{1b}[38;2;139;233;253m");
    }

    #[test]
    fn markdown_list_marker_uses_cyan() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("punctuation.list.markdown", "-")
            .expect("style should exist");

        assert_eq!(style.render().to_string(), "\u{1b}[38;2;139;233;253m");
    }

    #[test]
    fn markdown_plain_code_block_uses_orange() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.literal.block", "plain fenced block")
            .expect("style should exist");

        assert_eq!(style.render().to_string(), "\u{1b}[38;2;255;184;108m");
    }

    #[test]
    fn markdown_blockquote_uses_yellow_italic() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.quote", ">")
            .expect("style should exist");

        assert_eq!(
            style.render().to_string(),
            "\u{1b}[3m\u{1b}[38;2;241;250;140m"
        );
    }

    #[test]
    fn markdown_strikethrough_uses_comment_strike() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let style = theme
            .style_for("text.strikethrough", "strike")
            .expect("style should exist");

        assert_eq!(
            style.render().to_string(),
            "\u{1b}[9m\u{1b}[38;2;98;114;164m"
        );
    }
}
