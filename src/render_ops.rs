use serde::Serialize;

use crate::layout::LayoutDocument;
use crate::terminal::OSC8_CLOSE;
use crate::terminal::escape_control_sequences;
use crate::terminal::osc8_open;
use crate::theme::ColorMode;
use crate::theme::Theme;
use crate::theme::TokenStyle;
use crate::theme::TokenStyleSnapshot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderPlan {
    ops: Vec<RenderOp>,
}

impl RenderPlan {
    pub(crate) fn compile(layout: &LayoutDocument, theme: Theme) -> Self {
        let mut builder = RenderPlanBuilder::new(theme);

        for (row_index, row) in layout.rows().iter().enumerate() {
            compile_row(&mut builder, row);
            builder.reset_style();

            if row_index + 1 < layout.rows().len() {
                builder.push_newline();
            }
        }

        Self { ops: builder.ops }
    }

    pub(crate) fn compile_row(row: &crate::layout::LayoutRow, theme: Theme) -> Self {
        let mut builder = RenderPlanBuilder::new(theme);
        compile_row(&mut builder, row);
        builder.reset_hyperlink();
        builder.reset_style();
        Self { ops: builder.ops }
    }

    pub(crate) fn text(&self) -> String {
        self.ops
            .iter()
            .filter_map(|op| match op {
                RenderOp::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn encode(&self, theme: Theme, hyperlinks_enabled: bool) -> String {
        self.encode_marked(theme, hyperlinks_enabled, &[], None)
    }

    pub(crate) fn encode_marked(
        &self,
        theme: Theme,
        hyperlinks_enabled: bool,
        matches: &[std::ops::Range<usize>],
        current: Option<&std::ops::Range<usize>>,
    ) -> String {
        let mut rendered = String::new();
        let mut active_style = None;
        let mut active_hyperlink = false;
        let mut text_offset = 0;

        for op in &self.ops {
            match op {
                RenderOp::StartHyperlink { destination } => {
                    if hyperlinks_enabled {
                        rendered.push_str(&osc8_open(destination));
                        active_hyperlink = true;
                    }
                }
                RenderOp::EndHyperlink => {
                    if hyperlinks_enabled && active_hyperlink {
                        rendered.push_str(OSC8_CLOSE);
                        active_hyperlink = false;
                    }
                }
                RenderOp::SetStyle { style } => {
                    let transition = style.render_transition_from(active_style, theme.color_mode());
                    if !transition.is_empty() {
                        rendered.push_str(&transition);
                    }
                    active_style = Some(*style);
                }
                RenderOp::ResetStyle => {
                    if active_style.take().is_some() {
                        rendered.push_str("\x1b[0m");
                    }
                }
                RenderOp::Text { text } => {
                    let end = text_offset + text.len();
                    let mut cursor = text_offset;
                    for range in matches
                        .iter()
                        .filter(|range| range.start < end && range.end > text_offset)
                    {
                        let start = range.start.max(text_offset);
                        let stop = range.end.min(end);
                        rendered.push_str(&text[cursor - text_offset..start - text_offset]);
                        let selected = current == Some(range);
                        rendered.push_str(if selected { "\x1b[7;4m" } else { "\x1b[7m" });
                        rendered.push_str(&text[start - text_offset..stop - text_offset]);
                        rendered.push_str(if selected { "\x1b[24;27m" } else { "\x1b[27m" });
                        if selected && active_style.is_some_and(|style| style.is_underlined()) {
                            rendered.push_str("\x1b[4m");
                        }
                        cursor = stop;
                    }
                    rendered.push_str(&text[cursor - text_offset..]);
                    text_offset = end;
                }
                RenderOp::Newline => rendered.push('\n'),
            }
        }
        if hyperlinks_enabled && active_hyperlink {
            rendered.push_str(OSC8_CLOSE);
        }

        rendered
    }

    pub(crate) fn snapshot(&self, theme: Theme) -> RenderPlanSnapshot {
        RenderPlanSnapshot {
            color_mode: theme.color_mode(),
            ops: self
                .ops
                .iter()
                .map(|op| op.snapshot(theme.color_mode()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RenderOp {
    StartHyperlink { destination: String },
    EndHyperlink,
    SetStyle { style: TokenStyle },
    ResetStyle,
    Text { text: String },
    Newline,
}

impl RenderOp {
    fn snapshot(&self, color_mode: ColorMode) -> RenderOpSnapshot {
        match self {
            Self::StartHyperlink { destination } => RenderOpSnapshot::StartHyperlink {
                destination: destination.clone(),
            },
            Self::EndHyperlink => RenderOpSnapshot::EndHyperlink,
            Self::SetStyle { style } => RenderOpSnapshot::SetStyle {
                style: style.snapshot(color_mode),
            },
            Self::ResetStyle => RenderOpSnapshot::ResetStyle,
            Self::Text { text } => RenderOpSnapshot::Text {
                text: escape_control_sequences(text),
            },
            Self::Newline => RenderOpSnapshot::Newline,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RenderPlanSnapshot {
    pub color_mode: ColorMode,
    pub ops: Vec<RenderOpSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(crate) enum RenderOpSnapshot {
    StartHyperlink { destination: String },
    EndHyperlink,
    SetStyle { style: TokenStyleSnapshot },
    ResetStyle,
    Text { text: String },
    Newline,
}

struct RenderPlanBuilder {
    theme: Theme,
    active_style: Option<TokenStyle>,
    active_hyperlink: Option<String>,
    ops: Vec<RenderOp>,
}

impl RenderPlanBuilder {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            active_style: None,
            active_hyperlink: None,
            ops: Vec::new(),
        }
    }

    fn push_text(&mut self, text: &str, style: Option<TokenStyle>, hyperlink: Option<&str>) {
        self.set_hyperlink(hyperlink);
        let style = style.filter(|style| !style.renders_as_plain_text(self.theme.color_mode()));
        match style {
            Some(style) => {
                if self.active_style != Some(style) {
                    self.ops.push(RenderOp::SetStyle { style });
                    self.active_style = Some(style);
                }
            }
            None => self.reset_style(),
        }

        if !text.is_empty() {
            self.ops.push(RenderOp::Text {
                text: text.to_owned(),
            });
        }
    }

    fn reset_style(&mut self) {
        if self.active_style.take().is_some() {
            self.ops.push(RenderOp::ResetStyle);
        }
    }

    fn push_newline(&mut self) {
        self.ops.push(RenderOp::Newline);
    }

    fn set_hyperlink(&mut self, hyperlink: Option<&str>) {
        if self.active_hyperlink.as_deref() == hyperlink {
            return;
        }
        self.reset_hyperlink();
        if let Some(destination) = hyperlink {
            self.ops.push(RenderOp::StartHyperlink {
                destination: destination.to_owned(),
            });
            self.active_hyperlink = Some(destination.to_owned());
        }
    }

    fn reset_hyperlink(&mut self) {
        if self.active_hyperlink.take().is_some() {
            self.ops.push(RenderOp::EndHyperlink);
        }
    }
}

fn compile_row(builder: &mut RenderPlanBuilder, row: &crate::layout::LayoutRow) {
    let mut cell_index = 0usize;
    let mut column = 0usize;

    while cell_index < row.cells.len() || column < row.display_width {
        if row
            .cells
            .get(cell_index)
            .is_some_and(|cell| cell.column == column)
        {
            let cell = &row.cells[cell_index];
            let style = merged_style(cell.style, background_style_at(row, cell.column));
            builder.push_text(&cell.text, style, cell.hyperlink.as_deref());
            column += cell.width;
            cell_index += 1;
            continue;
        }

        let style = background_style_at(row, column);
        let run_end = next_cell_column(row, cell_index)
            .unwrap_or(row.display_width)
            .min(next_background_boundary(row, column).unwrap_or(row.display_width))
            .max(column + 1);
        builder.push_text(&" ".repeat(run_end - column), style, None);
        column = run_end;
    }
    builder.reset_hyperlink();
}

fn merged_style(
    foreground: Option<TokenStyle>,
    background: Option<TokenStyle>,
) -> Option<TokenStyle> {
    match (foreground, background) {
        (Some(foreground), Some(background)) => Some(foreground.with_background_under(background)),
        (Some(foreground), None) => Some(foreground),
        (None, Some(background)) => Some(background),
        (None, None) => None,
    }
}

fn background_style_at(row: &crate::layout::LayoutRow, column: usize) -> Option<TokenStyle> {
    row.background_runs
        .iter()
        .find(|run| run.start_column <= column && run.end_column > column)
        .map(|run| run.style)
}

fn next_background_boundary(row: &crate::layout::LayoutRow, column: usize) -> Option<usize> {
    row.background_runs
        .iter()
        .filter_map(|run| {
            if run.start_column > column {
                Some(run.start_column)
            } else if run.start_column <= column && run.end_column > column {
                Some(run.end_column)
            } else {
                None
            }
        })
        .min()
}

fn next_cell_column(row: &crate::layout::LayoutRow, cell_index: usize) -> Option<usize> {
    row.cells.get(cell_index).map(|cell| cell.column)
}
