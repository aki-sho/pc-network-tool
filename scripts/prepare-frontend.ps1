$ErrorActionPreference = 'Stop'

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$destination = Join-Path $root '.tauri-frontend'

New-Item -ItemType Directory -Path $destination -Force | Out-Null

Copy-Item -LiteralPath (Join-Path $root 'index.html') -Destination $destination -Force
foreach ($directory in @('renderer', 'services', 'styles', 'templates')) {
  Copy-Item -LiteralPath (Join-Path $root $directory) -Destination $destination -Recurse -Force
}
