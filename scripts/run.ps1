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

function Test-HttpOk {
    param([string]$Url)

    try {
        return (Invoke-WebRequest -UseBasicParsing -TimeoutSec 5 $Url).StatusCode -eq 200
    } catch {
        return $false
    }
}

function Get-LogTail {
    param(
        [string]$Path,
        [int]$Tail = 20
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        return ''
    }

    return ((Get-Content -LiteralPath $Path -Tail $Tail -ErrorAction SilentlyContinue) -join "`n")
}

function Get-WebUrlFromLogs {
    param(
        [string[]]$Paths,
        [string]$FallbackUrl
    )

    foreach ($path in $Paths) {
        if (-not (Test-Path -LiteralPath $path)) {
            continue
        }

        $content = Get-Content -LiteralPath $path -ErrorAction SilentlyContinue
        foreach ($line in $content) {
            if ($line -match 'Local:\s+(http://localhost:\d+)') {
                return $Matches[1]
            }
        }
    }

    return $FallbackUrl
}

function Get-ExistingWebPidFromStderr {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }

    $content = Get-Content -LiteralPath $Path -ErrorAction SilentlyContinue
    foreach ($line in $content) {
        if ($line -match 'PID:\s+(\d+)') {
            return [int]$Matches[1]
        }
    }

    return $null
}

function Test-WebReady {
    param([string]$Url)

    if (Test-HttpOk $Url) {
        return $true
    }

    $onboardingUrl = "$($Url.TrimEnd('/'))/onboarding/identity"
    if (Test-HttpOk $onboardingUrl) {
        return $true
    }

    return $false
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
$infraEnv = Read-EnvFile 'infra/.env'
$postgresUser = if ($infraEnv['POSTGRES_USER']) { $infraEnv['POSTGRES_USER'] } else { 'hexrelay' }
$postgresDb = if ($infraEnv['POSTGRES_DB']) { $infraEnv['POSTGRES_DB'] } else { 'hexrelay' }

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
    docker compose --env-file 'infra/.env' -f 'infra/docker-compose.yml' exec -T postgres pg_isready -U $postgresUser -d $postgresDb *> $null
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
        Test-HttpOk "$apiBaseUrl/health"
    }

    Write-Host '[run.ps1] Starting realtime service'
    $realtimeProcess = Start-CmdProcess -WorkingDirectory $root -EnvVars $realtimeEnv -Command 'cargo.exe run -p realtime-rs' -Name 'realtime-rs' -LogDir $logDir
    Wait-Until -Label 'realtime' -Probe {
        Test-HttpOk "$realtimeBaseUrl/health"
    }

    Write-Host '[run.ps1] Starting web dev server'
    $webEnv = @{
        'NEXT_PUBLIC_API_BASE_URL' = $apiBaseUrl
        'NEXT_PUBLIC_REALTIME_WS_URL' = $realtimeWsUrl
    }
    $webStdoutPath = Join-Path $logDir 'web.stdout.log'
    $webStderrPath = Join-Path $logDir 'web.stderr.log'
    $webBaseUrl = "http://localhost:$webPort"

    for ($attempt = 1; $attempt -le 2; $attempt++) {
        $webProcess = Start-WebProcess -Root $root -EnvVars $webEnv -Port $webPort -LogDir $logDir
        Wait-Until -Label 'web' -Probe {
            $stdoutTail = Get-LogTail -Path $webStdoutPath
            if ($stdoutTail -match 'Ready in') {
                return $true
            }

            $stderrTailInner = Get-LogTail -Path $webStderrPath
            if ($stderrTailInner -match 'Another next dev server is already running') {
                return $true
            }

            Test-HttpOk $webBaseUrl
        }

        $stderrTail = Get-LogTail -Path $webStderrPath
        if ($stderrTail -match 'Another next dev server is already running') {
            $existingWebPid = Get-ExistingWebPidFromStderr -Path $webStderrPath
            if ($existingWebPid -and $attempt -lt 2) {
                Write-Host "[run.ps1] Stopping stale Next dev server PID $existingWebPid and retrying web startup"
                try {
                    taskkill /PID $existingWebPid /T /F *> $null
                }
                catch {
                }
                Start-Sleep -Seconds 2
                continue
            }

            if ($existingWebPid) {
                throw "[run.ps1] Another Next dev server PID $existingWebPid is still running. Stop it and rerun npm run start."
            }

            throw '[run.ps1] Another Next dev server is already running, but its PID could not be determined. Stop it and rerun npm run start.'
        }
        else {
            $webBaseUrl = Get-WebUrlFromLogs -Paths @($webStdoutPath, $webStderrPath) -FallbackUrl $webBaseUrl
        }

        break
    }

    Wait-Until -Label 'web HTTP' -Probe {
        Test-WebReady -Url $webBaseUrl
    } -Attempts 60

    Write-Host ''
    Write-Host '[run.ps1] Local stack is ready'
    Write-Host "  API:      $apiBaseUrl"
    Write-Host "  Realtime: $realtimeBaseUrl"
    Write-Host "  Web:      $webBaseUrl"
    Write-Host "  Logs:     $logDir"
    Write-Host ''
    Write-Host '[run.ps1] Press Ctrl+C to stop the stack'

    $apiConsecutiveFailures = 0
    $realtimeConsecutiveFailures = 0
    $webConsecutiveFailures = 0
    while ($true) {
        if (Test-HttpOk "$apiBaseUrl/health") {
            $apiConsecutiveFailures = 0
        }
        else {
            $apiConsecutiveFailures += 1
            if ($apiConsecutiveFailures -ge 15) {
                throw '[run.ps1] API health check failed after startup'
            }
        }

        if (Test-HttpOk "$realtimeBaseUrl/health") {
            $realtimeConsecutiveFailures = 0
        }
        else {
            $realtimeConsecutiveFailures += 1
            if ($realtimeConsecutiveFailures -ge 15) {
                throw '[run.ps1] Realtime health check failed after startup'
            }
        }

        if (Test-WebReady -Url $webBaseUrl) {
            $webConsecutiveFailures = 0
        }
        else {
            $webConsecutiveFailures += 1
            if ($webConsecutiveFailures -ge 15) {
                throw '[run.ps1] Web health check failed after startup'
            }
        }

        Start-Sleep -Seconds 2
    }
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
