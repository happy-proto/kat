#!/usr/bin/env tclsh

proc render_theme {theme enabled} {
    set label "disabled"
    if {$enabled} {
        set label "enabled"
    }
    puts "$theme => $label"
}

foreach theme {Dracula Nord Monokai} {
    render_theme $theme [expr {$theme eq "Dracula"}]
}
