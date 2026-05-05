param(
    [switch]$Json,
    [Alias('h')]
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

if ($Help) {
    Write-Host 'Usage: status.ps1 [-Json]'
    exit 0
}

function Test-ProcessAlive {
    param([int]$ProcessId)

    if ($ProcessId -le 0) {
        return $false
    }

    return [bool](Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
}

function Test-HttpOk {
    param([string]$Url)

    try {
        return (Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 $Url).StatusCode -eq 200
    } catch {
        return $false
    }
}

function Test-WebReady {
    param([string]$Url)

    if (Test-HttpOk $Url) {
        return $true
    }

    return Test-HttpOk "$($Url.TrimEnd('/'))/onboarding/identity"
}

$root = Split-Path -Parent $PSScriptRoot
$statePath = Join-Path $root '.local-run\runtime-state.json'

if (-not (Test-Path -LiteralPath $statePath)) {
    if ($Json) {
        [pscustomobject]@{ active = $false; instances = @() } | ConvertTo-Json -Depth 6
    } else {
        Write-Host '[status.ps1] No tracked local runtime is active.'
    }
    exit 0
}

$state = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
$instances = @()
foreach ($instance in @($state.instances)) {
    $instances += [pscustomobject]@{
        id = $instance.id
        seedPersona = $instance.seedPersona
        apiPort = $instance.apiPort
        realtimePort = $instance.realtimePort
        webPort = $instance.webPort
        apiPid = $instance.apiPid
        realtimePid = $instance.realtimePid
        webPid = $instance.webPid
        apiProcessAlive = Test-ProcessAlive -ProcessId ([int]$instance.apiPid)
        realtimeProcessAlive = Test-ProcessAlive -ProcessId ([int]$instance.realtimePid)
        webProcessAlive = Test-ProcessAlive -ProcessId ([int]$instance.webPid)
        apiHealthy = Test-HttpOk "$($instance.apiUrl)/health"
        realtimeHealthy = Test-HttpOk "$($instance.realtimeUrl)/health"
        webHealthy = Test-WebReady -Url $instance.webUrl
        apiUrl = $instance.apiUrl
        realtimeUrl = $instance.realtimeUrl
        realtimeWsUrl = $instance.realtimeWsUrl
        webUrl = $instance.webUrl
        logDir = $instance.logDir
    }
}

$result = [pscustomobject]@{
    active = $true
    profile = $state.profile
    profilePath = $state.profilePath
    seedProfile = $state.seedProfile
    infraMode = $state.infraMode
    startedAt = $state.startedAt
    instances = $instances
}

if ($Json) {
    $result | ConvertTo-Json -Depth 8
    exit 0
}

Write-Host "[status.ps1] Runtime profile: $($result.profile)"
if ($result.seedProfile) {
    Write-Host "[status.ps1] Seed profile:    $($result.seedProfile)"
}
Write-Host "[status.ps1] Started at:      $($result.startedAt)"

foreach ($instance in $instances) {
    Write-Host ""
    Write-Host "[$($instance.id)]"
    Write-Host "  API:      pid=$($instance.apiPid) process=$($instance.apiProcessAlive) health=$($instance.apiHealthy) $($instance.apiUrl)"
    Write-Host "  Realtime: pid=$($instance.realtimePid) process=$($instance.realtimeProcessAlive) health=$($instance.realtimeHealthy) $($instance.realtimeUrl)"
    Write-Host "  Web:      pid=$($instance.webPid) process=$($instance.webProcessAlive) health=$($instance.webHealthy) $($instance.webUrl)"
    Write-Host "  WS:       $($instance.realtimeWsUrl)"
    Write-Host "  Logs:     $($instance.logDir)"
}
