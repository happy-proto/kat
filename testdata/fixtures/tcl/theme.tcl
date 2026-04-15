#!/usr/bin/env tclsh

proc render_theme {name enabled} {
    set status "disabled"
    if {$enabled} {
        set status "enabled"
    }
    puts "$name: $status"
}

render_theme "Dracula" true
