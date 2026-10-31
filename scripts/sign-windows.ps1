[CmdletBinding()]
param(
  [Parameter(Mandatory)][string]$CertificateThumbprint,
  [Parameter(Mandatory)][string]$FilePath,
  [string]$TimestampUrl = 'http://timestamp.digicert.com',
  [switch]$SelfSigned
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-signing.ps1')
$certificate = Get-ReleaseCertificate -Thumbprint $CertificateThumbprint
if (-not $SelfSigned -and $certificate.Subject -eq $certificate.Issuer) {
  throw 'Self-signed certificates require an explicitly labeled -SelfSigned build.'
}
if ($TimestampUrl -notmatch '^https?://') {
  throw 'An HTTP(S) timestamp server is required.'
}
$signature = Set-AuthenticodeSignature -LiteralPath $FilePath -Certificate $certificate `
  -HashAlgorithm SHA256 -IncludeChain All -TimestampServer $TimestampUrl
Assert-ReleaseSignature -LiteralPath $FilePath -Thumbprint $certificate.Thumbprint `
  -SelfSigned:$SelfSigned -RequireTimestamp | Out-Null
Write-Output "Signed: $FilePath"
