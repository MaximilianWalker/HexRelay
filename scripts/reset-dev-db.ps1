$ErrorActionPreference = 'Stop'

function Ensure-FileFromExample {
    param(
        [string]$Path,
        [string]$ExamplePath
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        Copy-Item -LiteralPath $ExamplePath -Destination $Path
    }
}

function Read-EnvFile {
    param([string]$Path)

    $values = @{}
    foreach ($line in Get-Content -LiteralPath $Path) {
        $trimmed = $line.Trim()
        if (-not $trimmed -or $trimmed.StartsWith('#')) {
            continue
        }

        $parts = $trimmed -split '=', 2
        if ($parts.Length -ne 2) {
            continue
        }

        $values[$parts[0].Trim()] = $parts[1].Trim()
    }

    return $values
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $cargoBin })) {
    $env:PATH = "$cargoBin;$($env:PATH)"
}

Ensure-FileFromExample -Path 'infra/.env' -ExamplePath 'infra/.env.example'
Ensure-FileFromExample -Path 'services/api-rs/.env' -ExamplePath 'services/api-rs/.env.example'

foreach ($envFile in @('infra/.env', 'services/api-rs/.env')) {
    $values = Read-EnvFile $envFile
    foreach ($entry in $values.GetEnumerator()) {
        [Environment]::SetEnvironmentVariable($entry.Key, $entry.Value, 'Process')
    }
    Write-Host "[reset-dev-db.ps1] Loaded env from $envFile"
}

function Convert-ResetArgs {
    param($RawArgs)

    $converted = @()
    $rawList = if ($null -eq $RawArgs) { @() } elseif ($RawArgs -is [array]) { $RawArgs } else { @($RawArgs) }
    for ($i = 0; $i -lt $rawList.Count; $i++) {
        $current = [string]$rawList[$i]
        switch ($current) {
            '-Profile' {
                $converted += '--profile'
                if ($i + 1 -lt $rawList.Count) {
                    $i += 1
                    $converted += [string]$rawList[$i]
                }
            }
            '-FixturesRoot' {
                $converted += '--fixtures-root'
                if ($i + 1 -lt $rawList.Count) {
                    $i += 1
                    $converted += [string]$rawList[$i]
                }
            }
            '-Json' { $converted += '--json' }
            '-Yes' { $converted += '--yes' }
            '-Help' { $converted += '--help' }
            '-' { $converted += '--help' }
            default { $converted += $current }
        }
    }

    return $converted
}

$resetArgs = @(Convert-ResetArgs -RawArgs @($args))
& cargo.exe run -p api-rs --bin reset_dev_db -- @resetArgs
exit $LASTEXITCODE
