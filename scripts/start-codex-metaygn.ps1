param(
    [string]$UserPrompt = "",
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

if (-not (Get-Command codex -ErrorAction SilentlyContinue)) {
    throw "codex CLI not found in PATH."
}

# If an HTTP daemon is running, stop it to avoid DB lock/contention with MCP mode.
$existingCli = Get-ChildItem -Path (Join-Path $repoRoot "target") -Filter "aletheia.exe" -Recurse -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if ($existingCli) {
    & $existingCli.FullName stop *> $null
}

# Ensure aletheia MCP server exists for Codex.
codex mcp get aletheia *> $null
if ($LASTEXITCODE -ne 0) {
    Write-Host "MCP server 'aletheia' missing. Installing..."
    & powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "install-codex.ps1")
}

$bootstrapPath = Join-Path $repoRoot "docs\codex-bootstrap-prompt.txt"
if (-not (Test-Path $bootstrapPath)) {
    throw "Missing bootstrap prompt: $bootstrapPath"
}

$bootstrap = Get-Content $bootstrapPath -Raw
if ($UserPrompt) {
    $bootstrap = "$bootstrap`n`nAdditional focus from user: $UserPrompt"
}

if ($NoLaunch) {
    Write-Host "Codex bootstrap prompt prepared:"
    Write-Host "-----"
    Write-Host $bootstrap
    Write-Host "-----"
    exit 0
}

codex $bootstrap
