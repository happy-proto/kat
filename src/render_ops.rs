use serde::Serialize;

use crate::{
    layout::LayoutDocument,
    terminal::escape_control_sequences,
    theme::{ColorMode, Theme, TokenStyle, TokenStyleSnapshot},
};

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

    pub(crate) fn encode(&self, theme: Theme) -> String {
        let mut rendered = String::new();
        let mut active_style = None;

        for op in &self.ops {
            match op {
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
                RenderOp::Text { text } => rendered.push_str(text),
                RenderOp::Newline => rendered.push('\n'),
            }
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
    SetStyle { style: TokenStyle },
    ResetStyle,
    Text { text: String },
    Newline,
}

impl RenderOp {
    fn snapshot(&self, color_mode: ColorMode) -> RenderOpSnapshot {
        match self {
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
    SetStyle { style: TokenStyleSnapshot },
    ResetStyle,
    Text { text: String },
    Newline,
}

struct RenderPlanBuilder {
    theme: Theme,
    active_style: Option<TokenStyle>,
    ops: Vec<RenderOp>,
}

impl RenderPlanBuilder {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            active_style: None,
            ops: Vec::new(),
        }
    }

    fn push_text(&mut self, text: &str, style: Option<TokenStyle>) {
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
}

fn compile_row(builder: &mut RenderPlanBuilder, row: &crate::layout::LayoutRow) {
    let mut cell_index = 0usize;
    let mut column = 0usize;

    while column < row.display_width {
        if row
            .cells
            .get(cell_index)
            .is_some_and(|cell| cell.column == column)
        {
            let cell = &row.cells[cell_index];
            let style = merged_style(cell.style, background_style_at(row, cell.column));
            builder.push_text(&cell.text, style);
            column += cell.width.max(1);
            cell_index += 1;
            continue;
        }

        let style = background_style_at(row, column);
        let run_end = next_cell_column(row, cell_index)
            .unwrap_or(row.display_width)
            .min(next_background_boundary(row, column).unwrap_or(row.display_width))
            .max(column + 1);
        builder.push_text(&" ".repeat(run_end - column), style);
        column = run_end;
    }
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
