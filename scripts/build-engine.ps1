# Configures, builds and tests the C++ engine on Windows, then stages the
# sidecar binary where the Tauri build expects it.
#
# Usage: scripts\build-engine.ps1 [-BuildType Release]

param(
    [string]$BuildType = "Release",
    [string]$Generator = ""
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$CmakeArgs = @("-S", ".", "-B", "build")
if ($Generator -ne "") {
    $CmakeArgs += @("-G", $Generator)
}
$CmakeArgs += @("-DCMAKE_BUILD_TYPE=$BuildType")

cmake @CmakeArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cmake --build build --config $BuildType
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

ctest --test-dir build --output-on-failure -C $BuildType
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "Engine ready. Sidecar staged under apps/desktop/src-tauri/binaries/"