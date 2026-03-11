# neco ローカル実行用（Windows PowerShell）
# 使用例: .\scripts_local\neco.ps1 "カレントディレクトリのファイル一覧"
# 温度指定: .\scripts_local\neco.ps1 -t 0.5 "ファイル一覧を表示"
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot\..\app
if ($args.Count -eq 0) {
    Write-Error "コマンドの説明を引数で指定してください。例: .\scripts_local\neco.ps1 ""ファイル一覧を表示"""
    exit 1
}
cargo run -q --bin neco -- @args
