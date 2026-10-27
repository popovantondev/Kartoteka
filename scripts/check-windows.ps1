& (Join-Path $PSScriptRoot 'build-noadmin.ps1') -CheckOnly
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
