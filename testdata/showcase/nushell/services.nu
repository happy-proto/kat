#!/usr/bin/env nu
# Nushell pipelines preserve records, tables, and typed values.
def summarize [minimum: filesize = 1kb, --enabled(-e)] {
    let services = [
        {name: 'api', enabled: true, size: 8kb}
        {name: 'worker', enabled: false, size: 2kb}
    ]
    $services
    | where size >= $minimum
    | where {|service| not $enabled or $service.enabled }
    | each {|service| $"服务：($service.name)" }
}

let raw_path = r#'/opt/example\data#archive'#
summarize 4kb --enabled | str join ', '
