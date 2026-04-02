set shell := ["cmd.exe", "/c"]

default:
    @echo off
    set TARGET=kat
    echo %TARGET%

pwsh:
    #!/usr/bin/env pwsh
    Write-Host "kat"
