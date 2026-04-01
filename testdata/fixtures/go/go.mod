module github.com/dcjanus/kat

go 1.24.0

toolchain go1.24.1

require (
	github.com/charmbracelet/lipgloss v1.0.0
	golang.org/x/tools v0.31.0
)

replace github.com/charmbracelet/lipgloss => ../forks/lipgloss

exclude golang.org/x/text v0.21.0

retract [v0.9.0, v0.9.2]
