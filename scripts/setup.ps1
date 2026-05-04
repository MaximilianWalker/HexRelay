$ErrorActionPreference = 'Stop'

function Invoke-Checked {
    param(
        [string]$Label,
        [scriptblock]$Command
    )

    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "[setup.ps1] $Label failed with exit code $LASTEXITCODE"
    }
}

function Ensure-FileFromExample {
    param(
        [string]$Path,
        [string]$ExamplePath
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        Copy-Item -LiteralPath $ExamplePath -Destination $Path
    }
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $cargoBin })) {
    $env:PATH = "$cargoBin;$($env:PATH)"
}

Ensure-FileFromExample -Path 'infra/.env' -ExamplePath 'infra/.env.example'

Write-Host '[setup.ps1] Installing web dependencies'
Invoke-Checked -Label 'npm ci --prefix apps/web' -Command {
    npm ci --prefix 'apps/web'
}

Write-Host '[setup.ps1] Fetching Rust dependencies'
Invoke-Checked -Label 'cargo fetch api-rs' -Command {
    cargo.exe fetch --manifest-path 'services/api-rs/Cargo.toml'
}
Invoke-Checked -Label 'cargo fetch realtime-rs' -Command {
    cargo.exe fetch --manifest-path 'services/realtime-rs/Cargo.toml'
}

Write-Host '[setup.ps1] Installing pinned security tooling'
Invoke-Checked -Label 'ensure-cargo-audit.ps1' -Command {
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File 'scripts/ensure-cargo-audit.ps1'
}

Write-Host '[setup.ps1] Complete'
