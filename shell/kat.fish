# kat shell integration
function kat --wraps=kat --description 'Browse files and open directories in the shell'
    set -l kat_cwd_file (mktemp -t kat-cwd.XXXXXX); or return
    command kat --cwd-file "$kat_cwd_file" $argv
    set -l kat_status $status
    if test $kat_status -eq 0; and test -s "$kat_cwd_file"
        set -l kat_directory (string collect < "$kat_cwd_file")
        if test -n "$kat_directory"
            builtin cd -- "$kat_directory"; or set kat_status $status
        end
    end
    command rm -f -- "$kat_cwd_file"
    return $kat_status
end
