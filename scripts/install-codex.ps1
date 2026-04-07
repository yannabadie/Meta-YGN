param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

if (-not (Get-Command codex -ErrorAction SilentlyContinue)) {
    throw "codex CLI not found in PATH."
}

# Stop daemon if running to avoid Windows file-lock errors during rebuild.
$existingCli = Get-ChildItem -Path (Join-Path $repoRoot "target") -Filter "aletheia.exe" -Recurse -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if ($existingCli) {
    & $existingCli.FullName stop *> $null
}

if ($Release) {
    cargo build -p metaygn-daemon -p metaygn-cli --release --features mcp
    $profile = "release"
}
else {
    cargo build -p metaygn-daemon -p metaygn-cli --features mcp
    $profile = "debug"
}

$bin = Get-ChildItem -Path (Join-Path $repoRoot "target") -Filter "aletheia.exe" -Recurse -ErrorAction Stop |
    Where-Object { $_.FullName -match "\\$profile\\aletheia\.exe$" } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if (-not $bin) {
    throw "aletheia binary not found under target/*/$profile/aletheia.exe"
}
$binPath = (Resolve-Path $bin.FullName).Path

if (codex mcp get aletheia *> $null) {
    codex mcp remove aletheia *> $null
}

codex mcp add aletheia -- $binPath mcp

Write-Host "MetaYGN registered as Codex MCP server 'aletheia'."
Write-Host "Verify with: codex mcp list"
