package kat.preview

data class ThemePreview(val name: String) {
    fun render(): String = "theme:$name"
}

fun main() {
    val preview = ThemePreview("Dracula")
    println(preview.render())
}
