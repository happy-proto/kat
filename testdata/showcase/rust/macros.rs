//! # Macro Showcase
//!
//! This file demonstrates nested Markdown highlighting inside Rustdoc.
//!
//! Rust macros such as `println!`, `format!`, and `vec!` should still keep
//! their Rust highlighting, while the doc comment keeps Markdown structure:
//!
//! - list items
//! - **bold text**
//! - _italic text_
//! - inline code like `BTreeMap<&str, &str>`
//! - block quotes that should still be treated as Markdown
//!
//! ```rust
//! let rendered = format!("theme={theme}");
//! println!("{rendered}");
//! ```
//!
//! ```bash
//! kat testdata/showcase/rust/macros.rs
//! ```
//!
//! > Rust code and Rustdoc Markdown should both stay readable.

use std::collections::BTreeMap;

/// Build a small map for the macro showcase.
///
/// This doc comment adds a few more Markdown constructs:
///
/// 1. Ordered lists.
/// 2. Fenced code blocks with language tags.
/// 3. `inline code` mixed with regular prose.
/// 4. Link syntax such as [kat](https://github.com/openai).
///
/// ```toml
/// [tool.kat]
/// theme = "Dracula"
/// ```
fn main() {
    let mut values = BTreeMap::new();
    values.insert("theme", "Dracula");
    values.insert("tool", "kat");
    values.insert("renderer", "tree-sitter");

    println!("{values:#?}");
}
