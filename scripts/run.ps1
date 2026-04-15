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

function Test-PortInUse {
    param([int]$Port)

    return [bool](Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue)
}

function Get-FreePort {
    param(
        [int]$PreferredPort,
        [System.Collections.Generic.HashSet[int]]$ReservedPorts
    )

    $port = $PreferredPort
    while ((Test-PortInUse -Port $port) -or ($null -ne $ReservedPorts -and $ReservedPorts.Contains($port))) {
        $port += 1
    }
    if ($null -ne $ReservedPorts) {
        [void]$ReservedPorts.Add($port)
    }
    return $port
}

function Wait-Until {
    param(
        [string]$Label,
        [scriptblock]$Probe,
        [int]$Attempts = 60,
        [int]$SleepSeconds = 1
    )

    for ($i = 0; $i -lt $Attempts; $i++) {
        if (& $Probe) {
            Write-Host "[run.ps1] $Label is ready"
            return
        }
        Start-Sleep -Seconds $SleepSeconds
    }

    throw "[run.ps1] $Label did not become ready after $Attempts attempts"
}

function Start-CmdProcess {
    param(
        [string]$WorkingDirectory,
        [hashtable]$EnvVars,
        [string]$Command,
        [string]$Name,
        [string]$LogDir
    )

    $launcherPath = Join-Path $LogDir "$Name.cmd"
    $stdoutPath = Join-Path $LogDir "$Name.stdout.log"
    $stderrPath = Join-Path $LogDir "$Name.stderr.log"

    $lines = @('@echo off')
    foreach ($entry in $EnvVars.GetEnumerator()) {
        $safeValue = $entry.Value -replace '"', '""'
        $lines += "set `"$($entry.Key)=$safeValue`""
    }
    $lines += $Command
    Set-Content -LiteralPath $launcherPath -Value $lines -Encoding Ascii

    return Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', $launcherPath -WorkingDirectory $WorkingDirectory -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath -WindowStyle Hidden -PassThru
}

function Start-WebProcess {
    param(
        [string]$Root,
        [hashtable]$EnvVars,
        [int]$Port,
        [string]$LogDir
    )

    $launcherPath = Join-Path $LogDir 'web.cmd'
    $stdoutPath = Join-Path $LogDir 'web.stdout.log'
    $stderrPath = Join-Path $LogDir 'web.stderr.log'
    $webDir = Join-Path $Root 'apps\web'

    $lines = @('@echo off', "cd /d `"$webDir`"")
    foreach ($entry in $EnvVars.GetEnumerator()) {
        $safeValue = $entry.Value -replace '"', '""'
        $lines += "set `"$($entry.Key)=$safeValue`""
    }
    $lines += ".\\node_modules\\.bin\\next.cmd dev --port $Port"
    Set-Content -LiteralPath $launcherPath -Value $lines -Encoding Ascii

    return Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', $launcherPath -WorkingDirectory $Root -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath -WindowStyle Hidden -PassThru
}

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $cargoBin })) {
    $env:PATH = "$cargoBin;$($env:PATH)"
}

Ensure-FileFromExample -Path 'infra/.env' -ExamplePath 'infra/.env.example'
Ensure-FileFromExample -Path 'services/api-rs/.env' -ExamplePath 'services/api-rs/.env.example'
Ensure-FileFromExample -Path 'services/realtime-rs/.env' -ExamplePath 'services/realtime-rs/.env.example'

$apiEnv = Read-EnvFile 'services/api-rs/.env'
$realtimeEnv = Read-EnvFile 'services/realtime-rs/.env'

$reservedPorts = [System.Collections.Generic.HashSet[int]]::new()
$apiPort = Get-FreePort -PreferredPort 18080 -ReservedPorts $reservedPorts
$realtimePort = Get-FreePort -PreferredPort 18081 -ReservedPorts $reservedPorts
$webPort = Get-FreePort -PreferredPort 3002 -ReservedPorts $reservedPorts

$apiBaseUrl = "http://127.0.0.1:$apiPort"
$realtimeBaseUrl = "http://127.0.0.1:$realtimePort"
$realtimeWsUrl = "ws://127.0.0.1:$realtimePort/ws"
$allowedOrigins = "http://localhost:$webPort,http://127.0.0.1:$webPort"

$apiEnv['API_BIND'] = "127.0.0.1:$apiPort"
$apiEnv['API_REALTIME_BASE_URL'] = $realtimeBaseUrl
$apiEnv['API_ALLOWED_ORIGINS'] = $allowedOrigins

$realtimeEnv['REALTIME_BIND'] = "127.0.0.1:$realtimePort"
$realtimeEnv['REALTIME_API_BASE_URL'] = $apiBaseUrl
$realtimeEnv['REALTIME_ALLOWED_ORIGINS'] = $allowedOrigins

$logDir = Join-Path $root '.local-run'
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

Write-Host '[run.ps1] Starting local infrastructure'
docker compose --env-file 'infra/.env' -f 'infra/docker-compose.yml' up -d postgres redis minio | Out-Null

Wait-Until -Label 'postgres' -Probe {
    docker compose --env-file 'infra/.env' -f 'infra/docker-compose.yml' exec -T postgres pg_isready -U 'hexrelay' -d 'hexrelay' *> $null
    $LASTEXITCODE -eq 0
}
Wait-Until -Label 'redis' -Probe {
    $result = docker compose --env-file 'infra/.env' -f 'infra/docker-compose.yml' exec -T redis redis-cli --raw ping 2>$null
    $result -match 'PONG'
}
Wait-Until -Label 'minio' -Probe {
    try {
        (Invoke-WebRequest -UseBasicParsing 'http://localhost:9000/minio/health/live').StatusCode -eq 200
    } catch {
        $false
    }
}

$apiProcess = $null
$realtimeProcess = $null
$webProcess = $null

try {
    Write-Host '[run.ps1] Starting API service'
    $apiProcess = Start-CmdProcess -WorkingDirectory $root -EnvVars $apiEnv -Command 'cargo.exe run -p api-rs' -Name 'api-rs' -LogDir $logDir
    Wait-Until -Label 'api' -Probe {
        try {
            (Invoke-WebRequest -UseBasicParsing "$apiBaseUrl/health").StatusCode -eq 200
        } catch {
            $false
        }
    }

    Write-Host '[run.ps1] Starting realtime service'
    $realtimeProcess = Start-CmdProcess -WorkingDirectory $root -EnvVars $realtimeEnv -Command 'cargo.exe run -p realtime-rs' -Name 'realtime-rs' -LogDir $logDir
    Wait-Until -Label 'realtime' -Probe {
        try {
            (Invoke-WebRequest -UseBasicParsing "$realtimeBaseUrl/health").StatusCode -eq 200
        } catch {
            $false
        }
    }

    Write-Host '[run.ps1] Starting web dev server'
    $webEnv = @{
        'NEXT_PUBLIC_API_BASE_URL' = $apiBaseUrl
        'NEXT_PUBLIC_REALTIME_WS_URL' = $realtimeWsUrl
    }
    $webProcess = Start-WebProcess -Root $root -EnvVars $webEnv -Port $webPort -LogDir $logDir
    Wait-Until -Label 'web' -Probe {
        try {
            (Invoke-WebRequest -UseBasicParsing "http://127.0.0.1:$webPort").StatusCode -eq 200
        } catch {
            $false
        }
    }

    Write-Host ''
    Write-Host '[run.ps1] Local stack is ready'
    Write-Host "  API:      $apiBaseUrl"
    Write-Host "  Realtime: $realtimeBaseUrl"
    Write-Host "  Web:      http://127.0.0.1:$webPort"
    Write-Host "  Logs:     $logDir"
    Write-Host ''
    Write-Host '[run.ps1] Press Ctrl+C to stop the stack'

    Wait-Process -Id $apiProcess.Id, $realtimeProcess.Id, $webProcess.Id
}
finally {
    foreach ($process in @($apiProcess, $realtimeProcess, $webProcess)) {
        if ($null -ne $process) {
            try {
                taskkill /PID $process.Id /T /F *> $null
            } catch {
            }
        }
    }
}
