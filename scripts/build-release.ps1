[CmdletBinding()]
param(
  [string]$CertificateThumbprint = $env:WINDOWS_CERTIFICATE_THUMBPRINT,
  [string]$TimestampUrl = 'http://timestamp.digicert.com',
  [switch]$SelfSigned
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-signing.ps1')
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$appConfiguration = Get-Content -LiteralPath (Join-Path $root 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
$version = $appConfiguration.version
$filePrefix = $appConfiguration.productName.Replace(' ', '-')
if ($version -notmatch '^\d+\.\d+\.\d+([-.][0-9A-Za-z.-]+)?$') {
  throw 'The application version is not suitable for a release filename.'
}
if ($SelfSigned -and -not $CertificateThumbprint) {
  $localConfiguration = Join-Path $root '.signing\certificate.json'
  if (-not (Test-Path -LiteralPath $localConfiguration)) {
    throw 'Run scripts/new-signing-certificate.ps1 first, then build with -SelfSigned.'
  }
  $CertificateThumbprint = (Get-Content -LiteralPath $localConfiguration -Raw | ConvertFrom-Json).thumbprint
}
if (-not $CertificateThumbprint) {
  throw 'Set WINDOWS_CERTIFICATE_THUMBPRINT or pass -CertificateThumbprint. Unsigned releases are refused.'
}
$certificate = Get-ReleaseCertificate -Thumbprint $CertificateThumbprint
if (-not $SelfSigned -and $certificate.Subject -eq $certificate.Issuer) {
  throw 'A self-signed certificate requires -SelfSigned; it is not a publicly trusted release.'
}

# Structured arguments preserve spaces and Unicode paths in Tauri's signing calls.
$powerShell = Join-Path ([Environment]::GetFolderPath('System')) 'WindowsPowerShell\v1.0\powershell.exe'
$signArguments = @('-NoProfile', '-NonInteractive', '-File', (Join-Path $PSScriptRoot 'sign-windows.ps1'),
  '-CertificateThumbprint', $certificate.Thumbprint, '-TimestampUrl', $TimestampUrl, '-FilePath', '%1')
if ($SelfSigned) { $signArguments += '-SelfSigned' }
$buildConfiguration = @{
  bundle = @{ windows = @{ signCommand = @{ cmd = $powerShell; args = $signArguments } } }
} | ConvertTo-Json -Depth 8 -Compress
$buildConfigurationPath = Join-Path $root 'src-tauri\target\release-signing-config.json'
New-Item -ItemType Directory -Path (Split-Path -Parent $buildConfigurationPath) -Force | Out-Null
[IO.File]::WriteAllText($buildConfigurationPath, $buildConfiguration, (New-Object Text.UTF8Encoding($false)))

$dist = Join-Path $root 'dist'
$targetRelease = Join-Path $root 'src-tauri\target\release'
$suffix = if ($SelfSigned) { '-SelfSigned' } else { '' }
$portableName = "$filePrefix-Portable-$version$suffix.exe"
$setupName = "$filePrefix-Setup-$version$suffix.exe"
$zipName = "$filePrefix-Portable-$version$suffix.zip"
$zipRootName = "$filePrefix-Portable-$version$suffix"
$stagingRoot = Join-Path $dist ('.staging-' + [Guid]::NewGuid().ToString('N'))
$stagingCreated = $false

Push-Location $root
try {
  npm run tauri:build -- --config $buildConfigurationPath -- --locked
  if ($LASTEXITCODE -ne 0) { throw "Tauri build failed with exit code $LASTEXITCODE" }

  $appExe = Join-Path $targetRelease "$($filePrefix.ToLowerInvariant()).exe"
  $installerName = "$($appConfiguration.productName)_${version}_x64-setup.exe"
  $installer = Join-Path $targetRelease "bundle\nsis\$installerName"
  Assert-ReleaseSignature -LiteralPath $installer -Thumbprint $certificate.Thumbprint `
    -SelfSigned:$SelfSigned -RequireTimestamp | Out-Null

  if (Test-Path -LiteralPath $dist) {
    if ((Get-Item -LiteralPath $dist -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
      throw 'The release directory must not be a junction or symbolic link.'
    }
  } else { New-Item -ItemType Directory -Path $dist | Out-Null }
  New-Item -ItemType Directory -Path $stagingRoot | Out-Null
  $stagingCreated = $true
  Copy-Item -LiteralPath $appExe -Destination (Join-Path $stagingRoot $portableName)
  Copy-Item -LiteralPath $installer -Destination (Join-Path $stagingRoot $setupName)
  # Bundling can restore the original unsigned EXE. Sign the final portable copy.
  $portableSignArguments = $signArguments | ForEach-Object {
    if ($_ -eq '%1') { Join-Path $stagingRoot $portableName } else { $_ }
  }
  & $powerShell @portableSignArguments
  if ($LASTEXITCODE -ne 0) { throw 'Signing the portable executable failed.' }
  foreach ($name in @($portableName, $setupName)) {
    Assert-ReleaseSignature -LiteralPath (Join-Path $stagingRoot $name) -Thumbprint $certificate.Thumbprint `
      -SelfSigned:$SelfSigned -RequireTimestamp | Out-Null
  }
  $zipRoot = Join-Path $stagingRoot $zipRootName
  New-Item -ItemType Directory -Path $zipRoot | Out-Null
  Copy-Item -LiteralPath (Join-Path $stagingRoot $portableName) -Destination $zipRoot
  $portableReadme = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'README-PORTABLE.txt') -Raw -Encoding UTF8
  $portableReadme = $portableReadme.Replace('{{VERSION}}', $version).Replace('{{EXECUTABLE}}', $portableName)
  Set-Content -LiteralPath (Join-Path $zipRoot 'README.txt') -Value $portableReadme -Encoding UTF8
  Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $zipRoot

  $publicCertificateName = "$filePrefix-Signing-$version$suffix.cer"
  Export-Certificate -Cert $certificate -FilePath (Join-Path $stagingRoot $publicCertificateName) | Out-Null
  Copy-Item -LiteralPath (Join-Path $stagingRoot $publicCertificateName) -Destination $zipRoot
  $signingNotice = @"
Certificate subject: $($certificate.Subject)
Certificate thumbprint: $($certificate.Thumbprint)
Self-signed development build: $([bool]$SelfSigned)
EXE signatures use SHA-256 and a timestamp.
$(if ($SelfSigned) { 'This certificate is self-signed. Windows does not trust this publisher by default. No Windows trust settings have been changed. The .cer file contains only the public certificate.' })
"@
  $noticeName = "$filePrefix-Signing-$version$suffix.txt"
  Set-Content -LiteralPath (Join-Path $stagingRoot $noticeName) -Value $signingNotice -Encoding UTF8
  Set-Content -LiteralPath (Join-Path $zipRoot 'SIGNING.txt') -Value $signingNotice -Encoding UTF8
  Compress-Archive -LiteralPath $zipRoot -DestinationPath (Join-Path $stagingRoot $zipName) -CompressionLevel Optimal

  foreach ($name in @($setupName, $portableName, $zipName)) {
    $artifact = Join-Path $stagingRoot $name
    $algorithm = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($artifact)
    try {
      $hash = ([BitConverter]::ToString($algorithm.ComputeHash($stream))).Replace('-', '').ToLowerInvariant()
    } finally {
      $stream.Dispose()
      $algorithm.Dispose()
    }
    Set-Content -LiteralPath "$artifact.sha256" -Value "$hash  $name" -Encoding ascii -NoNewline
  }
  # Publish verified files only; preserve other releases and PortableData.
  Get-ChildItem -LiteralPath $stagingRoot -File | ForEach-Object {
    Move-Item -LiteralPath $_.FullName -Destination (Join-Path $dist $_.Name) -Force
  }
  Write-Output "Signed release created in $dist"
} finally {
  Pop-Location
  if ($stagingCreated) {
    $resolvedStaging = (Resolve-Path -LiteralPath $stagingRoot).Path
    $expectedParent = [IO.Path]::GetFullPath($dist).TrimEnd('\') + '\'
    if (-not $resolvedStaging.StartsWith($expectedParent, [StringComparison]::OrdinalIgnoreCase) -or
        (Get-Item -LiteralPath $resolvedStaging -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
      throw 'Refusing to remove a staging directory outside the release directory.'
    }
    Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
  }
}
