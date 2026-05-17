param(
    [switch]$SkipServiceBackedTests
)

$ErrorActionPreference = 'Stop'

function Invoke-Checked {
    param(
        [string]$Label,
        [scriptblock]$Command
    )

    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "[test.ps1] $Label failed with exit code $LASTEXITCODE"
    }
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$previousSkipServiceBackedTests = $env:HEXRELAY_SKIP_SERVICE_BACKED_TESTS
if ($SkipServiceBackedTests) {
    Write-Host '[test.ps1] Skipping external service-backed Rust tests'
    $env:HEXRELAY_SKIP_SERVICE_BACKED_TESTS = '1'
}

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $cargoBin })) {
    $env:PATH = "$cargoBin;$($env:PATH)"
}

Write-Host '[test.ps1] Rust fmt/clippy/test'
Invoke-Checked -Label 'cargo fmt' -Command {
    cargo.exe fmt --all -- --check
}
Invoke-Checked -Label 'cargo clippy' -Command {
    cargo.exe clippy --all-targets --all-features -- -D warnings
}
Invoke-Checked -Label 'cargo test' -Command {
    cargo.exe test --all-features
}

Write-Host '[test.ps1] Web lint/test/build'
Invoke-Checked -Label 'web lint' -Command {
    npm run lint --prefix 'apps/web'
}
Invoke-Checked -Label 'web test coverage' -Command {
    npm run test:coverage --prefix 'apps/web'
}
Invoke-Checked -Label 'web build' -Command {
    npm run build --prefix 'apps/web'
}

if ($SkipServiceBackedTests) {
    if ($null -eq $previousSkipServiceBackedTests) {
        Remove-Item Env:\HEXRELAY_SKIP_SERVICE_BACKED_TESTS -ErrorAction SilentlyContinue
    } else {
        $env:HEXRELAY_SKIP_SERVICE_BACKED_TESTS = $previousSkipServiceBackedTests
    }
}

Write-Host '[test.ps1] Complete'
