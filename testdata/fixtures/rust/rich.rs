#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ThemePreview {
    value: String,
}

pub trait Renderable {
    fn render(&self) -> bool;
}

impl Renderable for ThemePreview {
    fn render(&self) -> bool {
        println!("{}", self.value);
        true
    }
}

macro_rules! themed {
    ($value:expr) => {
        ThemePreview { value: $value.into() }
    };
}

pub fn build() -> ThemePreview {
    let preview = themed!("Dracula");
    if preview.render() {
        preview
    } else {
        ThemePreview {
            value: String::from("fallback"),
        }
    }
}
