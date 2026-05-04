$ErrorActionPreference = 'Stop'

$requiredVersion = if ($env:CARGO_AUDIT_VERSION) { $env:CARGO_AUDIT_VERSION } else { '0.22.0' }

function Assert-NativeCommandSucceeded {
    param([string]$Label)

    if ($LASTEXITCODE -ne 0) {
        throw "[security.ps1] $Label failed with exit code $LASTEXITCODE"
    }
}

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
    Assert-NativeCommandSucceeded -Label 'cargo-audit install'
}

$finalVersion = & cargo-audit.exe --version
Assert-NativeCommandSucceeded -Label 'cargo-audit version check'
if ($finalVersion -notmatch "\b$([regex]::Escape($requiredVersion))\b") {
    throw "[security.ps1] Expected cargo-audit $requiredVersion but found $finalVersion"
}
Write-Host "[security.ps1] Using $finalVersion"
