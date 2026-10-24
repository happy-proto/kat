# Minimal virtual environment overlay, with structured environment values.
module warning {
    export-env {
        const file = path self
        error make -u {msg: $"($file) requires overlay use"}
    }
}
use warning

export-env {
    let virtual_env = '/opt/example'
    let path_name = if ('Path' in $env) { 'Path' } else { 'PATH' }
    let new_path = ($env | get $path_name | prepend ($virtual_env | path join 'bin'))
    load-env { $path_name: $new_path VIRTUAL_ENV: $virtual_env }
}
export alias deactivate = overlay hide activate
