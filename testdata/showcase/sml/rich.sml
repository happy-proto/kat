signature RENDERER =
sig
  val render : string -> unit
end

structure ThemePreview : RENDERER =
struct
  val name = "Dracula"

  fun render theme =
    let
      val message = theme ^ " preview"
    in
      print (message ^ "\n")
    end
end
