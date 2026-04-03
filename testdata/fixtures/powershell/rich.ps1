param([string]$Name = "kat")

function Invoke-Preview {
  if ($Name -eq "kat") {
    Write-Host "Ref $env:GITHUB_REF"
    Get-Item Env:PATH
  }
}

Invoke-Preview
