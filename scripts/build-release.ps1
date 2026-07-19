$ErrorActionPreference = 'Stop'

$version = '1.0.1'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$dist = Join-Path $root 'dist'
$targetRelease = Join-Path $root 'src-tauri\target\release'
$bundleNsis = Join-Path $targetRelease 'bundle\nsis'
$portableName = "PC-Network-Tool-Portable-$version.exe"
$setupName = "PC-Network-Tool-Setup-$version.exe"
$zipName = "PC-Network-Tool-Portable-$version.zip"
$zipRootName = "PC-Network-Tool-Portable-$version"
$stagingRoot = Join-Path $dist $zipRootName

Push-Location $root
try {
  npm run tauri:build
  if ($LASTEXITCODE -ne 0) {
    throw "Tauri build failed with exit code $LASTEXITCODE"
  }

  if (Test-Path -LiteralPath $dist) {
    Get-ChildItem -LiteralPath $dist -Force | Remove-Item -Recurse -Force
  } else {
    New-Item -ItemType Directory -Path $dist | Out-Null
  }

  $appExe = Join-Path $targetRelease 'pc-network-tool.exe'
  if (-not (Test-Path -LiteralPath $appExe)) {
    throw "Portable source executable was not found: $appExe"
  }

  $installer = Get-ChildItem -LiteralPath $bundleNsis -Filter '*.exe' |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1
  if (-not $installer) {
    throw "NSIS installer was not found in: $bundleNsis"
  }

  Copy-Item -LiteralPath $installer.FullName -Destination (Join-Path $dist $setupName)
  Copy-Item -LiteralPath $appExe -Destination (Join-Path $dist $portableName)

  New-Item -ItemType Directory -Path $stagingRoot | Out-Null
  Copy-Item -LiteralPath (Join-Path $dist $portableName) -Destination (Join-Path $stagingRoot $portableName)
  Copy-Item -LiteralPath (Join-Path $root 'scripts\README-PORTABLE.txt') -Destination (Join-Path $stagingRoot 'README.txt')

  Compress-Archive -LiteralPath $stagingRoot -DestinationPath (Join-Path $dist $zipName) -CompressionLevel Optimal
  Remove-Item -LiteralPath $stagingRoot -Recurse -Force

  foreach ($name in @($setupName, $portableName, $zipName)) {
    $artifact = Join-Path $dist $name
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::OpenRead($artifact)
    try {
      $hash = ([System.BitConverter]::ToString($sha256.ComputeHash($stream))).Replace('-', '').ToLowerInvariant()
    } finally {
      $stream.Dispose()
      $sha256.Dispose()
    }
    Set-Content -LiteralPath "$artifact.sha256" -Value "$hash  $name" -Encoding ascii -NoNewline
  }
} finally {
  Pop-Location
}
