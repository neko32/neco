# neco Windows インストール用スクリプト
# 使い方:
#   $env:NEKOKAN_BIN_DIR = "C:\nekokan\bin"
#   .\scripts_local\install_neco.ps1
$ErrorActionPreference = "Stop"

if (-not $env:NEKOKAN_BIN_DIR) {
    Write-Error "環境変数 NEKOKAN_BIN_DIR が未設定です。例: `$env:NEKOKAN_BIN_DIR = ""C:\nekokan\bin"""
    exit 1
}

$repoRoot = Join-Path $PSScriptRoot ".."
$appDir = Join-Path $repoRoot "app"

Set-Location $appDir
cargo build --release --bin neco

$sourceExe = Join-Path $appDir "target\release\neco.exe"
if (-not (Test-Path $sourceExe)) {
    Write-Error "ビルド成果物が見つかりません: $sourceExe"
    exit 1
}

$installDir = Join-Path $env:NEKOKAN_BIN_DIR "neco"
New-Item -ItemType Directory -Path $installDir -Force | Out-Null

$destExe = Join-Path $installDir "neco.exe"
Copy-Item -Path $sourceExe -Destination $destExe -Force

Write-Output "installed: $destExe"
