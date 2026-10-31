[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$signingDirectory = Join-Path $root '.signing'
$configurationPath = Join-Path $signingDirectory 'certificate.json'

if (Test-Path -LiteralPath $configurationPath) {
  $configuration = Get-Content -LiteralPath $configurationPath -Raw | ConvertFrom-Json
  $certificate = Get-Item -LiteralPath "Cert:\CurrentUser\My\$($configuration.thumbprint)" -ErrorAction SilentlyContinue
  if ($certificate -and $certificate.HasPrivateKey -and $certificate.NotAfter -gt (Get-Date).AddDays(7)) {
    Write-Output "Existing self-signed certificate: $($certificate.Thumbprint)"
    return
  }
}

# Keep the private key in Windows; do not change Trusted Root/Publisher stores.
$certificate = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject 'CN=PC Network Tool Local Development' `
  -FriendlyName 'PC Network Tool self-signed development' `
  -CertStoreLocation 'Cert:\CurrentUser\My' `
  -KeyAlgorithm RSA `
  -KeyLength 3072 `
  -HashAlgorithm SHA256 `
  -KeyExportPolicy NonExportable `
  -NotAfter (Get-Date).AddYears(1)

New-Item -ItemType Directory -Path $signingDirectory -Force | Out-Null
Export-Certificate -Cert $certificate -FilePath (Join-Path $signingDirectory 'certificate.cer') | Out-Null
@{
  thumbprint = $certificate.Thumbprint
  subject = $certificate.Subject
  selfSigned = $true
  expires = $certificate.NotAfter.ToUniversalTime().ToString('o')
} | ConvertTo-Json | Set-Content -LiteralPath $configurationPath -Encoding UTF8
Write-Output "Created self-signed certificate: $($certificate.Thumbprint)"
Write-Output 'Only the public certificate was exported. Windows trust settings were not changed.'
