param(
    [string]$RuntimeProfile = 'single',
    [string]$SeedProfile = '',
    [Alias('h')]
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

if ($Help) {
    Write-Host 'Usage: run.ps1 [-RuntimeProfile single|dual|triple|path] [-SeedProfile dm-basic]'
    Write-Host 'Default startup uses the clean single profile and does not seed fixture data.'
    exit 0
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

function Copy-EnvTable {
    param([hashtable]$Source)

    $copy = @{}
    foreach ($entry in $Source.GetEnumerator()) {
        $copy[$entry.Key] = $entry.Value
    }
    return $copy
}

function Set-ProcessEnvFromTable {
    param([hashtable]$Values)

    foreach ($entry in $Values.GetEnumerator()) {
        [Environment]::SetEnvironmentVariable($entry.Key, $entry.Value, 'Process')
    }
}

function Read-RuntimeProfile {
    param([string]$Profile)

    $profileJson = & node 'scripts/validate-runtime-profiles.mjs' '--print' $Profile
    if ($LASTEXITCODE -ne 0) {
        throw "[run.ps1] runtime profile '$Profile' failed validation"
    }

    return ($profileJson | ConvertFrom-Json)
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
        if ($port -ge 65535) {
            throw "[run.ps1] no available TCP port at or above $PreferredPort"
        }
        $port += 1
    }
    if ($null -ne $ReservedPorts) {
        [void]$ReservedPorts.Add($port)
    }
    return $port
}

function Test-ProcessAlive {
    param([int]$ProcessId)

    if ($ProcessId -le 0) {
        return $false
    }

    return [bool](Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
}

function Stop-ProcessTree {
    param(
        [int]$ProcessId,
        [string]$ExpectedLauncherPath = ''
    )

    if (Test-ProcessAlive -ProcessId $ProcessId) {
        if ($ExpectedLauncherPath -and -not (Test-ProcessCommandLineContains -ProcessId $ProcessId -ExpectedPath $ExpectedLauncherPath)) {
            return
        }
        try {
            taskkill /PID $ProcessId /T /F *> $null
        } catch {
        }
    }
}

function Test-ProcessCommandLineContains {
    param(
        [int]$ProcessId,
        [string]$ExpectedPath
    )

    $process = Get-CimInstance Win32_Process -Filter "ProcessId = $ProcessId" -ErrorAction SilentlyContinue
    if ($null -eq $process -or -not $process.CommandLine) {
        return $false
    }

    return $process.CommandLine.Contains($ExpectedPath)
}

function Wait-Until {
    param(
        [string]$Label,
        [scriptblock]$Probe,
        [int]$Attempts = 60,
        [int]$SleepSeconds = 1,
        [scriptblock]$FailureProbe = $null,
        [scriptblock]$OnFailure = $null
    )

    for ($i = 0; $i -lt $Attempts; $i++) {
        if (& $Probe) {
            Write-Host "[run.ps1] $Label is ready"
            return
        }
        if ($null -ne $FailureProbe -and (& $FailureProbe)) {
            if ($null -ne $OnFailure) {
                & $OnFailure
            }
            throw "[run.ps1] $Label failed before becoming ready"
        }
        Start-Sleep -Seconds $SleepSeconds
    }

    if ($null -ne $OnFailure) {
        & $OnFailure
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

function Write-StartupLogTail {
    param(
        [string]$Label,
        [string]$StdoutPath,
        [string]$StderrPath,
        [int]$Tail = 40
    )

    Write-Host "[run.ps1] $Label did not become ready. Recent logs:"
    foreach ($log in @(
        @{ Name = 'stdout'; Path = $StdoutPath },
        @{ Name = 'stderr'; Path = $StderrPath }
    )) {
        Write-Host "[run.ps1] $Label $($log.Name): $($log.Path)"
        $tailOutput = Get-LogTail -Path $log.Path -Tail $Tail
        if ($tailOutput) {
            Write-Host $tailOutput
        } else {
            Write-Host '[run.ps1] (no log output)'
        }
    }
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
    $runtimeInstance = $EnvVars['HEXRELAY_RUNTIME_INSTANCE']

    if ($runtimeInstance) {
        $runtimeTsConfigDir = Join-Path $webDir '.runtime-tsconfig'
        New-Item -ItemType Directory -Force -Path $runtimeTsConfigDir | Out-Null
        $runtimeTsConfig = [ordered]@{
            extends = '../tsconfig.json'
            include = @('../next-env.d.ts', '../**/*.ts', '../**/*.tsx', "../.next-$runtimeInstance/types/**/*.ts", "../.next-$runtimeInstance/dev/types/**/*.ts", '../**/*.mts')
            exclude = @('../node_modules')
        }
        $runtimeTsConfig | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $runtimeTsConfigDir "$runtimeInstance.json") -Encoding Ascii
    }

    $lines = @('@echo off', "cd /d `"$webDir`"")
    foreach ($entry in $EnvVars.GetEnumerator()) {
        $safeValue = $entry.Value -replace '"', '""'
        $lines += "set `"$($entry.Key)=$safeValue`""
    }
    $lines += ".\\node_modules\\.bin\\next.cmd dev --port $Port"
    Set-Content -LiteralPath $launcherPath -Value $lines -Encoding Ascii

    return Start-Process -FilePath 'cmd.exe' -ArgumentList '/c', $launcherPath -WorkingDirectory $Root -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath -WindowStyle Hidden -PassThru
}

function Write-RuntimeState {
    param(
        [string]$StatePath,
        [object]$State
    )

    $State | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $StatePath -Encoding UTF8
}

function Get-RuntimeState {
    param([string]$StatePath)

    if (-not (Test-Path -LiteralPath $StatePath)) {
        return $null
    }

    return (Get-Content -LiteralPath $StatePath -Raw | ConvertFrom-Json)
}

function Test-StateHasLiveProcesses {
    param([object]$State)

    if ($null -eq $State -or $null -eq $State.instances) {
        return $false
    }

    foreach ($instance in @($State.instances)) {
        foreach ($property in @('apiPid', 'realtimePid', 'webPid')) {
            $value = $instance.$property
            if ($null -ne $value -and (Test-ProcessAlive -ProcessId ([int]$value))) {
                return $true
            }
        }
    }

    return $false
}

function Start-RuntimeInstance {
    param(
        [object]$Instance,
        [hashtable]$BaseApiEnv,
        [hashtable]$BaseRealtimeEnv,
        [string]$Root,
        [string]$RunDir,
        [System.Collections.Generic.HashSet[int]]$ReservedPorts,
        [System.Collections.IList]$StartedProcesses
    )

    $instanceId = [string]$Instance.id
    $apiPort = Get-FreePort -PreferredPort ([int]$Instance.apiPort) -ReservedPorts $ReservedPorts
    $realtimePort = Get-FreePort -PreferredPort ([int]$Instance.realtimePort) -ReservedPorts $ReservedPorts
    $webPort = Get-FreePort -PreferredPort ([int]$Instance.webPort) -ReservedPorts $ReservedPorts

    if (($apiPort -ne [int]$Instance.apiPort) -or ($realtimePort -ne [int]$Instance.realtimePort) -or ($webPort -ne [int]$Instance.webPort)) {
        Write-Host "[run.ps1] $instanceId requested ports were unavailable; using api=$apiPort realtime=$realtimePort web=$webPort"
    }

    $apiBaseUrl = "http://127.0.0.1:$apiPort"
    $realtimeBaseUrl = "http://127.0.0.1:$realtimePort"
    $realtimeWsUrl = "ws://127.0.0.1:$realtimePort/ws"
    $allowedOrigins = "http://localhost:$webPort,http://127.0.0.1:$webPort"
    $instanceLogDir = Join-Path $RunDir $instanceId
    New-Item -ItemType Directory -Force -Path $instanceLogDir | Out-Null

    $apiEnv = Copy-EnvTable $BaseApiEnv
    $apiEnv['API_BIND'] = "127.0.0.1:$apiPort"
    $apiEnv['API_REALTIME_BASE_URL'] = $realtimeBaseUrl
    $apiEnv['API_ALLOWED_ORIGINS'] = $allowedOrigins

    $realtimeEnv = Copy-EnvTable $BaseRealtimeEnv
    $realtimeEnv['REALTIME_BIND'] = "127.0.0.1:$realtimePort"
    $realtimeEnv['REALTIME_API_BASE_URL'] = $apiBaseUrl
    $realtimeEnv['REALTIME_ALLOWED_ORIGINS'] = $allowedOrigins
    $realtimeEnv['REALTIME_ENABLE_DEV_FAULTS'] = 'true'

    $apiStdoutPath = Join-Path $instanceLogDir 'api-rs.stdout.log'
    $apiStderrPath = Join-Path $instanceLogDir 'api-rs.stderr.log'
    Write-Host "[run.ps1] Starting $instanceId API service"
    $apiProcess = Start-CmdProcess -WorkingDirectory $Root -EnvVars $apiEnv -Command 'cargo.exe run -p api-rs --bin api-rs' -Name 'api-rs' -LogDir $instanceLogDir
    [void]$StartedProcesses.Add($apiProcess)
    Wait-Until -Label "$instanceId api" -Probe {
        Test-HttpOk "$apiBaseUrl/health"
    } -FailureProbe {
        -not (Test-ProcessAlive -ProcessId $apiProcess.Id)
    } -OnFailure {
        Write-StartupLogTail -Label "$instanceId API" -StdoutPath $apiStdoutPath -StderrPath $apiStderrPath
    }

    $realtimeStdoutPath = Join-Path $instanceLogDir 'realtime-rs.stdout.log'
    $realtimeStderrPath = Join-Path $instanceLogDir 'realtime-rs.stderr.log'
    Write-Host "[run.ps1] Starting $instanceId realtime service"
    $realtimeProcess = Start-CmdProcess -WorkingDirectory $Root -EnvVars $realtimeEnv -Command 'cargo.exe run -p realtime-rs' -Name 'realtime-rs' -LogDir $instanceLogDir
    [void]$StartedProcesses.Add($realtimeProcess)
    Wait-Until -Label "$instanceId realtime" -Probe {
        Test-HttpOk "$realtimeBaseUrl/health"
    } -FailureProbe {
        -not (Test-ProcessAlive -ProcessId $realtimeProcess.Id)
    } -OnFailure {
        Write-StartupLogTail -Label "$instanceId realtime" -StdoutPath $realtimeStdoutPath -StderrPath $realtimeStderrPath
    }

    Write-Host "[run.ps1] Starting $instanceId web dev server"
    $webEnv = @{
        'HEXRELAY_RUNTIME_INSTANCE' = $instanceId
        'NEXT_PUBLIC_API_BASE_URL' = $apiBaseUrl
        'NEXT_PUBLIC_REALTIME_WS_URL' = $realtimeWsUrl
    }
    $webStdoutPath = Join-Path $instanceLogDir 'web.stdout.log'
    $webStderrPath = Join-Path $instanceLogDir 'web.stderr.log'
    $webBaseUrl = "http://localhost:$webPort"

    for ($attempt = 1; $attempt -le 2; $attempt++) {
        $webProcess = Start-WebProcess -Root $Root -EnvVars $webEnv -Port $webPort -LogDir $instanceLogDir
        [void]$StartedProcesses.Add($webProcess)
        Wait-Until -Label "$instanceId web" -Probe {
            $stdoutTail = Get-LogTail -Path $webStdoutPath
            if ($stdoutTail -match 'Ready in') {
                return $true
            }

            $stderrTailInner = Get-LogTail -Path $webStderrPath
            if ($stderrTailInner -match 'Another next dev server is already running') {
                return $true
            }

            Test-HttpOk $webBaseUrl
        } -FailureProbe {
            $stderrTailInner = Get-LogTail -Path $webStderrPath
            if ($stderrTailInner -match 'Another next dev server is already running') {
                return $false
            }

            -not (Test-ProcessAlive -ProcessId $webProcess.Id)
        } -OnFailure {
            Write-StartupLogTail -Label "$instanceId web" -StdoutPath $webStdoutPath -StderrPath $webStderrPath
        }

        $stderrTail = Get-LogTail -Path $webStderrPath
        if ($stderrTail -match 'Another next dev server is already running') {
            $existingWebPid = Get-ExistingWebPidFromStderr -Path $webStderrPath
            if ($existingWebPid -and $attempt -lt 2) {
                Write-Host "[run.ps1] Stopping stale Next dev server PID $existingWebPid and retrying $instanceId web startup"
                Stop-ProcessTree -ProcessId $existingWebPid
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

    Wait-Until -Label "$instanceId web HTTP" -Probe {
        Test-WebReady -Url $webBaseUrl
    } -Attempts 60 -FailureProbe {
        -not (Test-ProcessAlive -ProcessId $webProcess.Id)
    } -OnFailure {
        Write-StartupLogTail -Label "$instanceId web HTTP" -StdoutPath $webStdoutPath -StderrPath $webStderrPath
    }

    return [pscustomobject]@{
        id = $instanceId
        seedPersona = $Instance.seedPersona
        apiPort = $apiPort
        realtimePort = $realtimePort
        webPort = $webPort
        apiPid = $apiProcess.Id
        realtimePid = $realtimeProcess.Id
        webPid = $webProcess.Id
        apiLauncher = (Join-Path $instanceLogDir 'api-rs.cmd')
        realtimeLauncher = (Join-Path $instanceLogDir 'realtime-rs.cmd')
        webLauncher = (Join-Path $instanceLogDir 'web.cmd')
        apiUrl = $apiBaseUrl
        realtimeUrl = $realtimeBaseUrl
        realtimeWsUrl = $realtimeWsUrl
        webUrl = $webBaseUrl
        logDir = $instanceLogDir
        realtimeInternalToken = if ($realtimeEnv['REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN']) { $realtimeEnv['REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN'] } else { 'hexrelay-dev-channel-dispatch-token-change-me' }
    }
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
Set-ProcessEnvFromTable $infraEnv
Set-ProcessEnvFromTable $apiEnv

$postgresUser = if ($infraEnv['POSTGRES_USER']) { $infraEnv['POSTGRES_USER'] } else { 'hexrelay' }
$postgresDb = if ($infraEnv['POSTGRES_DB']) { $infraEnv['POSTGRES_DB'] } else { 'hexrelay' }
$profile = Read-RuntimeProfile -Profile $RuntimeProfile

$runDir = Join-Path $root '.local-run'
$statePath = Join-Path $runDir 'runtime-state.json'
New-Item -ItemType Directory -Force -Path $runDir | Out-Null

$existingState = Get-RuntimeState -StatePath $statePath
if (Test-StateHasLiveProcesses -State $existingState) {
    throw "[run.ps1] A tracked local runtime is already active. Run scripts/status.ps1 or scripts/stop.ps1 before starting another profile."
}
if ($null -ne $existingState) {
    Remove-Item -LiteralPath $statePath -Force
}

Write-Host "[run.ps1] Starting local infrastructure"
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

if ($SeedProfile.Trim()) {
    Write-Host "[run.ps1] Seeding local database with '$SeedProfile'"
    $seedStdoutPath = Join-Path $runDir 'seed.stdout.json'
    $seedStderrPath = Join-Path $runDir 'seed.stderr.log'
    $seedProcess = Start-Process -FilePath 'cargo.exe' -ArgumentList @('run', '-p', 'api-rs', '--bin', 'seed_dev', '--', '--profile', $SeedProfile, '--json') -WorkingDirectory $root -RedirectStandardOutput $seedStdoutPath -RedirectStandardError $seedStderrPath -WindowStyle Hidden -Wait -PassThru
    if ($seedProcess.ExitCode -ne 0) {
        if (Test-Path -LiteralPath $seedStderrPath) {
            Get-Content -LiteralPath $seedStderrPath -Tail 40
        }
        throw "[run.ps1] Seed profile '$SeedProfile' failed"
    }
    Write-Host "[run.ps1] Seed output written to $seedStdoutPath"
}

$reservedPorts = [System.Collections.Generic.HashSet[int]]::new()
$startedProcesses = New-Object System.Collections.ArrayList
$stateInstances = @()
$state = [pscustomobject]@{
    profile = $profile.name
    profilePath = $profile.profilePath
    seedProfile = if ($SeedProfile.Trim()) { $SeedProfile } else { $null }
    infraMode = $profile.infraMode
    startedAt = (Get-Date).ToUniversalTime().ToString('o')
    root = $root
    instances = $stateInstances
}

try {
    foreach ($instance in @($profile.instances)) {
        $instanceState = Start-RuntimeInstance -Instance $instance -BaseApiEnv $apiEnv -BaseRealtimeEnv $realtimeEnv -Root $root -RunDir $runDir -ReservedPorts $reservedPorts -StartedProcesses $startedProcesses
        $stateInstances += $instanceState
        $state.instances = $stateInstances
        Write-RuntimeState -StatePath $statePath -State $state
    }

    Write-Host ''
    Write-Host "[run.ps1] Local runtime profile '$($profile.name)' is ready"
    foreach ($instance in $stateInstances) {
        Write-Host "  [$($instance.id)] API:      $($instance.apiUrl)"
        Write-Host "  [$($instance.id)] Realtime: $($instance.realtimeUrl)"
        Write-Host "  [$($instance.id)] WS:       $($instance.realtimeWsUrl)"
        Write-Host "  [$($instance.id)] Web:      $($instance.webUrl)"
        Write-Host "  [$($instance.id)] Logs:     $($instance.logDir)"
    }
    Write-Host ''
    Write-Host '[run.ps1] Use scripts/status.ps1 from another shell to inspect health.'
    Write-Host '[run.ps1] Press Ctrl+C or run scripts/stop.ps1 to stop tracked processes.'

    $failureCounts = @{}
    foreach ($instance in $stateInstances) {
        $failureCounts["$($instance.id):api"] = 0
        $failureCounts["$($instance.id):realtime"] = 0
        $failureCounts["$($instance.id):web"] = 0
    }

    while ($true) {
        foreach ($instance in $stateInstances) {
            $checks = @(
                @{ Key = "$($instance.id):api"; Label = "$($instance.id) API"; Ok = (Test-HttpOk "$($instance.apiUrl)/health") },
                @{ Key = "$($instance.id):realtime"; Label = "$($instance.id) realtime"; Ok = (Test-HttpOk "$($instance.realtimeUrl)/health") },
                @{ Key = "$($instance.id):web"; Label = "$($instance.id) web"; Ok = (Test-WebReady -Url $instance.webUrl) }
            )

            foreach ($check in $checks) {
                if ($check.Ok) {
                    $failureCounts[$check.Key] = 0
                }
                else {
                    $failureCounts[$check.Key] += 1
                    if ($failureCounts[$check.Key] -ge 15) {
                        throw "[run.ps1] $($check.Label) health check failed after startup"
                    }
                }
            }
        }

        Start-Sleep -Seconds 2
    }
}
finally {
    foreach ($process in $startedProcesses) {
        if ($null -ne $process) {
            Stop-ProcessTree -ProcessId $process.Id
        }
    }
    if (Test-Path -LiteralPath $statePath) {
        Remove-Item -LiteralPath $statePath -Force
    }
}
