//! # Guide
//!
//! Rustdoc should support fenced Markdown code blocks and let them reuse the
//! same nested runtimes as top-level Markdown.
//!
//! ```rust
//! fn nested() -> i32 {
//!     42
//! }
//! ```
//!
//! ```python
//! class Nested:
//!     def render(self) -> int:
//!         return 42
//! ```
pub fn answer() -> i32 {
    42
}
