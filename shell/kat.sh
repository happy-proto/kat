# kat shell integration
kat() {
    local kat_cwd_file kat_status kat_directory
    kat_cwd_file="$(mktemp -t kat-cwd.XXXXXX)" || return
    command kat --cwd-file "$kat_cwd_file" "$@"
    kat_status=$?
    if [ "$kat_status" -eq 0 ] && [ -s "$kat_cwd_file" ]; then
        kat_directory="$(cat -- "$kat_cwd_file")"
        if [ -n "$kat_directory" ]; then
            builtin cd -- "$kat_directory" || kat_status=$?
        fi
    fi
    command rm -f -- "$kat_cwd_file"
    return "$kat_status"
}
