use std::env;
use std::ops::Range;
use std::time::Duration;

use terminal_colorsaurus::{QueryOptions, background_color};

const RESET: &str = "\x1b[0m";
const FG: &str = "\x1b[38;2;248;248;242m";
const MUTED: &str = "\x1b[38;2;98;114;164m";
const ACCENT: &str = "\x1b[38;2;139;233;253m";
const BOUNDARY: &str = "\x1b[38;2;80;250;123m";
const DRACULA_BACKGROUND: Rgb = Rgb(40, 42, 54);
const DRACULA_SELECTION: Rgb = Rgb(68, 71, 90);
const FALLBACK_TINT_BG: Rgb = Rgb(48, 52, 68);

#[derive(Clone, Copy)]
struct Rgb(u8, u8, u8);

impl Rgb {
    fn to_bg_escape(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.0, self.1, self.2)
    }

    fn mix(self, other: Self, weight: f32) -> Self {
        fn mix_channel(base: u8, other: u8, weight: f32) -> u8 {
            let base = base as f32;
            let other = other as f32;
            (base + (other - base) * weight).round().clamp(0.0, 255.0) as u8
        }

        Self(
            mix_channel(self.0, other.0, weight),
            mix_channel(self.1, other.1, weight),
            mix_channel(self.2, other.2, weight),
        )
    }

    fn distance(self, other: Self) -> f32 {
        let dr = self.0 as f32 - other.0 as f32;
        let dg = self.1 as f32 - other.1 as f32;
        let db = self.2 as f32 - other.2 as f32;
        (dr * dr + dg * dg + db * db).sqrt()
    }
}

#[derive(Clone, Copy)]
struct TintPalette {
    normal: Rgb,
}

#[derive(Clone, Copy)]
enum VisualMode {
    Tint,
}

impl VisualMode {
    fn label(self) -> &'static str {
        match self {
            Self::Tint => "tint",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::Tint => "Neutral Dracula-style background tint on nested lines.",
        }
    }
}

struct DemoCase {
    id: &'static str,
    title: &'static str,
    note: &'static str,
    lines: &'static [&'static str],
    nested_regions: &'static [Range<usize>],
}

fn main() {
    let filters: Vec<String> = env::args().skip(1).collect();
    let tint_palette = detect_tint_palette();
    let cases = demo_cases();
    let filtered: Vec<&DemoCase> = if filters.is_empty() {
        cases.iter().collect()
    } else {
        cases.iter()
            .filter(|case| {
                filters
                    .iter()
                    .any(|filter| case.id.contains(filter) || case.title.contains(filter))
            })
            .collect()
    };

    if filtered.is_empty() {
        eprintln!("No demo scenarios matched the given filter.");
        std::process::exit(1);
    }

    print_intro(tint_palette);
    for (index, case) in filtered.iter().enumerate() {
        if index > 0 {
            println!();
        }
        render_case(case, tint_palette);
    }
}

fn print_intro(tint_palette: TintPalette) {
    println!(
        "{ACCENT}Nested Region Visual Demo{RESET}\n{MUTED}Temporary renderer mock. This demo isolates region contrast only and does not depend on real parser output.{RESET}\n"
    );
    println!(
        "{MUTED}Mode:{RESET} {ACCENT}{:<14}{RESET} {}",
        VisualMode::Tint.label(),
        VisualMode::Tint.summary()
    );
    println!(
        "\n{MUTED}Tint palette:{RESET} normal=rgb({},{},{})",
        tint_palette.normal.0,
        tint_palette.normal.1,
        tint_palette.normal.2
    );
}

fn render_case(case: &DemoCase, tint_palette: TintPalette) {
    let divider = "=".repeat(88);
    println!("{ACCENT}{divider}{RESET}");
    println!("{ACCENT}Scenario:{RESET} {} ({})", case.title, case.id);
    println!("{MUTED}{}{RESET}", case.note);
    println!("{ACCENT}{divider}{RESET}");
    println!();
    println!("{ACCENT}[{}]{RESET} {}", VisualMode::Tint.label(), VisualMode::Tint.summary());
    println!(
        "{}",
        render_block(case.lines, case.nested_regions, VisualMode::Tint, tint_palette)
    );
}

fn render_block(
    lines: &[&str],
    nested_regions: &[Range<usize>],
    mode: VisualMode,
    tint_palette: TintPalette,
) -> String {
    let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let mut rendered = String::new();

    for (index, line) in lines.iter().enumerate() {
        let in_nested = nested_regions
            .iter()
            .any(|range| index >= range.start && index < range.end);
        let marker = match mode {
            VisualMode::Tint => "  ",
        };

        if matches!(mode, VisualMode::Tint) && in_nested {
            rendered.push_str(BOUNDARY);
            rendered.push_str(marker);
            rendered.push_str(RESET);
            rendered.push(' ');
            rendered.push_str(&tint_palette.normal.to_bg_escape());
            rendered.push_str(FG);
            rendered.push_str(&format!("{line:<width$}"));
            rendered.push_str(RESET);
        } else {
            rendered.push_str(MUTED);
            rendered.push_str(marker);
            rendered.push_str(RESET);
            rendered.push(' ');
            rendered.push_str(FG);
            rendered.push_str(line);
            rendered.push_str(RESET);
        }

        if index + 1 < lines.len() {
            rendered.push('\n');
        }
    }

    rendered
}

fn detect_tint_palette() -> TintPalette {
    let mut options = QueryOptions::default();
    options.timeout = Duration::from_millis(120);

    match background_color(options) {
        Ok(color) => {
            let (r, g, b) = color.scale_to_8bit();
            let background = Rgb(r, g, b);
            let neutral_target = if background.distance(DRACULA_BACKGROUND) < 24.0 {
                DRACULA_SELECTION
            } else {
                background.mix(DRACULA_SELECTION, 0.65)
            };
            TintPalette {
                normal: background.mix(neutral_target, 0.28),
            }
        }
        Err(_) => TintPalette {
            normal: FALLBACK_TINT_BG,
        },
    }
}

fn demo_cases() -> &'static [DemoCase] {
    &[
        DemoCase {
            id: "github-actions-run",
            title: "GitHub Actions run blocks",
            note: "One YAML host with several child runtimes. This is the block scenario called out in the issue.",
            lines: &[
                "name: Advanced CI",
                "",
                "jobs:",
                "  build:",
                "    runs-on: ubuntu-latest",
                "    steps:",
                "      - name: Default Shell Step",
                "        run: |",
                "          set -euo pipefail",
                "          echo \"Target ${{ matrix.target }}\"",
                "      - name: Python Template Step",
                "        shell: python {0}",
                "        run: |",
                "          print(\"${{ github.ref_name }}\")",
                "      - name: PowerShell Step",
                "        shell: pwsh",
                "        run: |",
                "          function Invoke-Preview { Write-Host \"Ref $env:GITHUB_REF\" }",
                "          Invoke-Preview",
            ],
            nested_regions: &[8..10, 13..14, 17..19],
        },
        DemoCase {
            id: "justfile-recipe",
            title: "Justfile recipe body",
            note: "Recipe header is the host. The indented shell body is the nested runtime region.",
            lines: &[
                "set shell := [\"bash\", \"-cu\"]",
                "",
                "deploy profile=\"staging\" target=\"x86_64-unknown-linux-gnu\":",
                "    @echo \"Preparing {{profile}} deploy for {{target}}\"",
                "    cargo build --target {{target}}",
                "    if [ -f .env ]; then",
                "      source .env",
                "    fi",
                "    ./scripts/deploy.sh \"{{profile}}\" \"{{target}}\"",
                "",
                "lint:",
                "    cargo fmt --check",
            ],
            nested_regions: &[3..9, 11..12],
        },
        DemoCase {
            id: "markdown-fence",
            title: "Markdown fenced code block",
            note: "Markdown prose is the host. The fenced code block is the child runtime region.",
            lines: &[
                "# Kat Theme Preview",
                "",
                "This Markdown file keeps a fenced block for renderer experiments.",
                "",
                "```rust",
                "fn preview(theme: &str) -> String {",
                "    format!(\"theme={theme}\")",
                "}",
                "```",
                "",
                "Back in Markdown prose after the nested block.",
            ],
            nested_regions: &[4..9],
        },
    ]
}
