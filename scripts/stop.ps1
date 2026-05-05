param(
    [string]$RuntimeProfile = '',
    [switch]$Json,
    [Alias('h')]
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

if ($Help) {
    Write-Host 'Usage: stop.ps1 [-RuntimeProfile single|dual|triple] [-Json]'
    exit 0
}

function Test-ProcessAlive {
    param([int]$ProcessId)

    if ($ProcessId -le 0) {
        return $false
    }

    return [bool](Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
}

function Stop-ProcessTree {
    param([int]$ProcessId)

    if (Test-ProcessAlive -ProcessId $ProcessId) {
        try {
            taskkill /PID $ProcessId /T /F *> $null
            return $true
        } catch {
            return $false
        }
    }

    return $false
}

$root = Split-Path -Parent $PSScriptRoot
$statePath = Join-Path $root '.local-run\runtime-state.json'

if (-not (Test-Path -LiteralPath $statePath)) {
    if ($Json) {
        [pscustomobject]@{ stopped = @(); message = 'no tracked local runtime is active' } | ConvertTo-Json -Depth 6
    } else {
        Write-Host '[stop.ps1] No tracked local runtime is active.'
    }
    exit 0
}

$state = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
if ($RuntimeProfile.Trim() -and $state.profile -ne $RuntimeProfile) {
    throw "[stop.ps1] Active runtime profile is '$($state.profile)', not '$RuntimeProfile'."
}

$stopped = @()
foreach ($instance in @($state.instances)) {
    foreach ($entry in @(
        @{ Name = 'api'; Pid = $instance.apiPid },
        @{ Name = 'realtime'; Pid = $instance.realtimePid },
        @{ Name = 'web'; Pid = $instance.webPid }
    )) {
        $pidValue = [int]$entry.Pid
        $wasStopped = Stop-ProcessTree -ProcessId $pidValue
        $stopped += [pscustomobject]@{
            instanceId = $instance.id
            service = $entry.Name
            pid = $pidValue
            stopped = $wasStopped
        }
    }
}

Remove-Item -LiteralPath $statePath -Force

if ($Json) {
    [pscustomobject]@{ profile = $state.profile; stopped = $stopped } | ConvertTo-Json -Depth 8
    exit 0
}

Write-Host "[stop.ps1] Stopped tracked local runtime profile '$($state.profile)'."
foreach ($entry in $stopped) {
    Write-Host "  [$($entry.instanceId)] $($entry.service) pid=$($entry.pid) stopped=$($entry.stopped)"
}
