param(
    [Parameter(Mandatory=$true)][string]$NsisPath,
    [string]$ProjectPath=(Split-Path -Parent $PSScriptRoot),
    [string]$OutputPath
)
$ErrorActionPreference='Stop'
if(-not(Test-Path -LiteralPath $NsisPath -PathType Leaf)){throw 'Укажите путь к makensis.exe через -NsisPath.'}
$appDir=Join-Path $ProjectPath 'dist'
foreach($name in @('Kartoteka.exe','libc++.dll','libunwind.dll','libwinpthread-1.dll')){
    if(-not(Test-Path -LiteralPath (Join-Path $appDir $name))){throw "Сначала выполните build-noadmin.ps1: нет $name"}
}
$noticeSource=Join-Path $ProjectPath 'third-party'
if(-not(Test-Path -LiteralPath $noticeSource)){throw 'Missing third-party notices.'}
$noticeDest=Join-Path $appDir 'licenses'
New-Item -ItemType Directory -Path $noticeDest -Force | Out-Null
Copy-Item -Path (Join-Path $noticeSource '*') -Destination $noticeDest -Recurse -Force
$buildRoot=if($env:CARGO_TARGET_DIR){$env:CARGO_TARGET_DIR}else{Join-Path $env:LOCALAPPDATA 'Popovantondev\KartotekaBuildGNU_LLVM'}
$icon=Get-ChildItem -LiteralPath (Join-Path $buildRoot 'x86_64-pc-windows-gnullvm\release\build') -Recurse -Filter kartoteka.ico -File | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if(-not $icon){throw 'Нет значка из release-сборки.'}
if(-not $OutputPath){$OutputPath=Join-Path $ProjectPath 'installer-dist\Kartoteka-0.5.0-Setup.exe'}
$OutputPath=[IO.Path]::GetFullPath($OutputPath)
New-Item -ItemType Directory -Path (Split-Path -Parent $OutputPath) -Force | Out-Null
$setupArgs=@('/V3',"/DAPPDIR=$appDir","/DAPPICON=$($icon.FullName)","/DOUTFILE=$OutputPath",(Join-Path $ProjectPath 'installer\Kartoteka.nsi'))
& $NsisPath @setupArgs
if($LASTEXITCODE -ne 0){throw "NSIS завершился с кодом $LASTEXITCODE"}
if(-not(Test-Path -LiteralPath $OutputPath)){throw 'Установщик не создан.'}
$hash=(Get-FileHash -LiteralPath $OutputPath -Algorithm SHA256).Hash.ToLower()
($hash+'  '+(Split-Path -Leaf $OutputPath)) | Set-Content -LiteralPath ($OutputPath+'.sha256') -Encoding ascii
Write-Host "Готов установщик без прав администратора: $OutputPath"

