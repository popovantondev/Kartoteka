param(
    [string]$ProjectPath = (Split-Path -Parent $PSScriptRoot),
    [switch]$CheckOnly,
    [switch]$DebugBuild
)
$ErrorActionPreference = 'Stop'
$toolchain = 'stable-x86_64-pc-windows-gnullvm'
$target = 'x86_64-pc-windows-gnullvm'
$rustupHome = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { Join-Path $env:USERPROFILE '.rustup' }
$toolBin = Join-Path $rustupHome "toolchains\$toolchain\bin"
$cargo = Join-Path $toolBin 'cargo.exe'
if (-not (Test-Path -LiteralPath $cargo)) { throw "Rust $toolchain не найден. Установка не выполнялась." }
$toolsRoot = Join-Path $env:LOCALAPPDATA 'Popovantondev\KartotekaTools'
$compiler = Get-ChildItem -LiteralPath $toolsRoot -Filter 'x86_64-w64-mingw32-clang.exe' -Recurse -File | Select-Object -First 1
if (-not $compiler) { throw "LLVM-MinGW не найден в $toolsRoot" }
$llvmBin = $compiler.Directory.FullName
$env:PATH = "$toolBin;$llvmBin;$env:PATH"
$env:RUSTUP_TOOLCHAIN = $toolchain
$env:RUSTC = Join-Path $toolBin 'rustc.exe'
$env:RUSTDOC = Join-Path $toolBin 'rustdoc.exe'
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER = $compiler.FullName
$env:CC = $compiler.FullName
$env:AR = Join-Path $llvmBin 'llvm-ar.exe'
$env:CARGO_TARGET_DIR = Join-Path $env:LOCALAPPDATA 'Popovantondev\KartotekaBuildGNU_LLVM'
$logRoot = Join-Path $ProjectPath 'build-logs'
New-Item -ItemType Directory -Path $logRoot -Force | Out-Null
$log = Join-Path $logRoot ("verify-{0}.txt" -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
function Invoke-CargoStep([string[]]$CargoArguments) {
    Write-Host ("cargo " + ($CargoArguments -join ' '))
    $previousErrorAction = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    & $cargo @CargoArguments 2>&1 | Tee-Object -FilePath $log -Append | ForEach-Object { Write-Host $_ }
    $cargoExitCode = $LASTEXITCODE
    $ErrorActionPreference = $previousErrorAction
    if ($cargoExitCode -ne 0) { throw "Проверка завершилась с кодом $cargoExitCode. Журнал: $log" }
}
Push-Location -LiteralPath $ProjectPath
try {
    Invoke-CargoStep @('fmt','--all','--','--check')
    Invoke-CargoStep @('check','--locked','--target',$target)
    Invoke-CargoStep @('test','--locked','--target',$target)
    if (-not $CheckOnly) {
        $buildArgs = @('build','--locked','--target',$target)
        $profile = 'debug'
        if (-not $DebugBuild) { $buildArgs += '--release'; $profile = 'release' }
        Invoke-CargoStep $buildArgs
        $binary = Join-Path $env:CARGO_TARGET_DIR "$target\$profile\kartoteka.exe"
        if (-not (Test-Path -LiteralPath $binary)) { throw "EXE не найден: $binary" }
        $distribution = Join-Path $ProjectPath 'dist'
        New-Item -ItemType Directory -Path $distribution -Force | Out-Null
        Copy-Item -LiteralPath $binary -Destination (Join-Path $distribution 'Kartoteka.exe')
        foreach ($dll in @('libc++.dll','libunwind.dll','libwinpthread-1.dll')) {
            $dllPath = Join-Path $llvmBin $dll
            if (Test-Path -LiteralPath $dllPath) { Copy-Item -LiteralPath $dllPath -Destination $distribution }
        }
        Write-Host "Готово: $distribution"
    }
    Write-Host "Проверки завершены. Журнал: $log"
} finally { Pop-Location }
