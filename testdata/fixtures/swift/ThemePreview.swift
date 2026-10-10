import Foundation

let optionalTheme: String? = nil

@available(*, deprecated)
struct ThemePreview {
    static let defaultTheme = "Dracula"

    func render(name: String) -> String {
        "Preview \(name)"
    }
}

print(ThemePreview().render(name: "kat"))
