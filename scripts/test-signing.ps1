[CmdletBinding()]
param([string]$FilePath)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-signing.ps1')
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$configuration = Get-Content -LiteralPath (Join-Path $root '.signing\certificate.json') -Raw | ConvertFrom-Json
if (-not $FilePath) {
  $appConfiguration = Get-Content -LiteralPath (Join-Path $root 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
  $filePrefix = $appConfiguration.productName.Replace(' ', '-')
  $FilePath = Join-Path $root "dist\$filePrefix-Portable-$($appConfiguration.version)-SelfSigned.exe"
}
$FilePath = (Resolve-Path -LiteralPath $FilePath).Path
$thumbprint = $configuration.thumbprint

function Assert-Rejected {
  param([string]$Description, [scriptblock]$Action)
  $rejected = $false
  try { & $Action | Out-Null } catch { $rejected = $true }
  if (-not $rejected) { throw "FAIL: $Description" }
  Write-Output "PASS: $Description"
}

Assert-ReleaseSignature -LiteralPath $FilePath -Thumbprint $thumbprint -SelfSigned -RequireTimestamp | Out-Null
Write-Output 'PASS: expected self-signed signature and timestamp'
Assert-Rejected 'self-signed file refused without explicit opt-in' {
  Assert-ReleaseSignature -LiteralPath $FilePath -Thumbprint $thumbprint -RequireTimestamp
}
Assert-Rejected 'unexpected signer refused' {
  Assert-ReleaseSignature -LiteralPath $FilePath -Thumbprint ('0' * 40) -SelfSigned -RequireTimestamp
}

$temporaryDirectory = Join-Path $root ('.signing\verification-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporaryDirectory | Out-Null
$tampered = Join-Path $temporaryDirectory 'tampered.exe'
$unsigned = Join-Path $temporaryDirectory 'unsigned.exe'
try {
  [IO.File]::WriteAllBytes($unsigned, (New-Object byte[] 1024))
  Assert-Rejected 'unsigned file refused' {
    Assert-ReleaseSignature -LiteralPath $unsigned -Thumbprint $thumbprint -SelfSigned
  }
  $bytes = [IO.File]::ReadAllBytes($FilePath)
  if ($bytes.Length -le 4096) { throw 'The test fixture is too small.' }
  $bytes[4096] = $bytes[4096] -bxor 1
  [IO.File]::WriteAllBytes($tampered, $bytes)
  Assert-Rejected 'modified EXE refused despite the self-signed exception' {
    Assert-ReleaseSignature -LiteralPath $tampered -Thumbprint $thumbprint -SelfSigned -RequireTimestamp
  }
} finally {
  foreach ($temporaryFile in @($tampered, $unsigned)) {
    if (Test-Path -LiteralPath $temporaryFile) { Remove-Item -LiteralPath $temporaryFile -Force }
  }
  Remove-Item -LiteralPath $temporaryDirectory
}
