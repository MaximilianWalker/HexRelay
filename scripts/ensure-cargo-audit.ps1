$ErrorActionPreference = 'Stop'

$requiredVersion = if ($env:CARGO_AUDIT_VERSION) { $env:CARGO_AUDIT_VERSION } else { '0.22.0' }

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $cargoBin })) {
    $env:PATH = "$cargoBin;$($env:PATH)"
}

$installedVersion = ''
try {
    $versionOutput = & cargo-audit.exe --version 2>$null
    if ($LASTEXITCODE -eq 0 -and $versionOutput -match '([0-9]+\.[0-9]+\.[0-9]+)') {
        $installedVersion = $Matches[1]
    }
} catch {
}

if ($installedVersion -ne $requiredVersion) {
    Write-Host "[security.ps1] Installing cargo-audit $requiredVersion"
    & cargo.exe install cargo-audit --version $requiredVersion --locked
}

$finalVersion = & cargo-audit.exe --version
Write-Host "[security.ps1] Using $finalVersion"
