$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSHOME 'Modules\Microsoft.PowerShell.Security\Microsoft.PowerShell.Security.psd1')

function Get-ReleaseCertificate {
  param([Parameter(Mandatory)][string]$Thumbprint)
  if ($Thumbprint -notmatch '^[0-9a-fA-F]{40}$') {
    throw 'A 40-character certificate thumbprint is required.'
  }
  $certificate = Get-Item -LiteralPath "Cert:\CurrentUser\My\$Thumbprint" -ErrorAction Stop
  if (-not $certificate.HasPrivateKey) {
    throw 'The code signing certificate has no accessible private key.'
  }
  $now = Get-Date
  if ($certificate.NotBefore -gt $now -or $certificate.NotAfter -le $now) {
    throw 'The code signing certificate is not currently valid.'
  }
  $usages = $certificate.Extensions | Where-Object { $_.Oid.Value -eq '2.5.29.37' } |
    ForEach-Object { $_.EnhancedKeyUsages }
  if (-not ($usages | Where-Object { $_.Value -eq '1.3.6.1.5.5.7.3.3' })) {
    throw 'The certificate is not authorized for code signing.'
  }
  return $certificate
}

function Assert-ReleaseSignature {
  param(
    [Parameter(Mandatory)][string]$LiteralPath,
    [Parameter(Mandatory)][string]$Thumbprint,
    [switch]$SelfSigned,
    [switch]$RequireTimestamp
  )
  $signature = Get-AuthenticodeSignature -LiteralPath $LiteralPath
  if (-not $signature.SignerCertificate -or $signature.SignerCertificate.Thumbprint -ne $Thumbprint) {
    throw "The signer does not match the selected certificate: $LiteralPath"
  }
  if ($signature.Status -ne 'Valid') {
    if (-not $SelfSigned -or
        $signature.SignerCertificate.Subject -ne $signature.SignerCertificate.Issuer) {
      throw "Signature verification failed ($($signature.Status)): $LiteralPath -- $($signature.StatusMessage)"
    }
    if (-not ('PcNetworkTool.AuthenticodeVerification' -as [type])) {
      Add-Type -TypeDefinition (Get-Content -LiteralPath (Join-Path $PSScriptRoot 'AuthenticodeVerification.cs') -Raw)
    }
    $trustError = [PcNetworkTool.AuthenticodeVerification]::Verify((Get-Item -LiteralPath $LiteralPath).FullName)
    # CERT_E_UNTRUSTEDROOT only. Do not accept arbitrary UnknownError results.
    if ($trustError -ne -2146762487) {
      throw "Authenticode verification failed ($trustError): $LiteralPath"
    }
  }
  if ($RequireTimestamp -and -not $signature.TimeStamperCertificate) {
    throw "The signature has no timestamp: $LiteralPath"
  }
  return $signature
}
