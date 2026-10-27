# Совместимый вход для прежней инструкции
param([switch]$CheckOnly)
& (Join-Path $PSScriptRoot 'build-noadmin.ps1') -CheckOnly:$CheckOnly
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
