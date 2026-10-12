module ThemePreview

/// The active terminal theme.
type Theme =
    member Name: string
    member Render: width: int -> string

val render: theme: Theme -> width: int -> string
