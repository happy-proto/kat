module github.com/example/kat

go 1.24.0

toolchain go1.24.1

require (
	github.com/charmbracelet/lipgloss v1.0.0
	github.com/muesli/termenv v0.15.2
	golang.org/x/tools v0.31.0
)

replace (
	github.com/charmbracelet/lipgloss => ../forks/lipgloss
	github.com/muesli/termenv v0.15.2 => github.com/example/termenv v0.15.2-preview.1
)

exclude golang.org/x/text v0.21.0

retract (
	[v0.8.0, v0.8.3]
	v0.9.1
)
