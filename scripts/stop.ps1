param(
    [string]$RuntimeProfile = '',
    [switch]$Json,
    [Alias('h')]
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

if ($Help) {
    Write-Host 'Usage: stop.ps1 [-RuntimeProfile single|dual|triple|path] [-Json]'
    exit 0
}

function Test-ProcessAlive {
    param([int]$ProcessId)

    if ($ProcessId -le 0) {
        return $false
    }

    return [bool](Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
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

function Stop-ProcessTree {
    param(
        [int]$ProcessId,
        [string]$ExpectedLauncherPath
    )

    if (Test-ProcessAlive -ProcessId $ProcessId) {
        if ($ExpectedLauncherPath -and -not (Test-ProcessCommandLineContains -ProcessId $ProcessId -ExpectedPath $ExpectedLauncherPath)) {
            return $false
        }
        try {
            taskkill /PID $ProcessId /T /F *> $null
            return $true
        } catch {
            return $false
        }
    }

    return $false
}

function Read-RuntimeProfileSpec {
    param([string]$Profile)

    $validatorPath = Join-Path $root 'scripts\validate-runtime-profiles.mjs'
    $profileJson = & node $validatorPath '--print' $Profile 2>$null
    if ($LASTEXITCODE -ne 0) {
        return $null
    }

    return ($profileJson | ConvertFrom-Json)
}

function Test-RuntimeProfileMatches {
    param(
        [string]$Profile,
        [object]$State
    )

    if (-not $Profile.Trim()) {
        return $true
    }
    if ($State.profile -eq $Profile -or $State.profilePath -eq $Profile) {
        return $true
    }

    $resolved = Read-RuntimeProfileSpec -Profile $Profile
    return $null -ne $resolved -and ($State.profile -eq $resolved.name -or $State.profilePath -eq $resolved.profilePath)
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
if (-not (Test-RuntimeProfileMatches -Profile $RuntimeProfile -State $state)) {
    throw "[stop.ps1] Active runtime profile is '$($state.profile)', not '$RuntimeProfile'."
}

$stopped = @()
foreach ($instance in @($state.instances)) {
    foreach ($entry in @(
        @{ Name = 'api'; Pid = $instance.apiPid; Launcher = $instance.apiLauncher },
        @{ Name = 'realtime'; Pid = $instance.realtimePid; Launcher = $instance.realtimeLauncher },
        @{ Name = 'web'; Pid = $instance.webPid; Launcher = $instance.webLauncher }
    )) {
        $pidValue = [int]$entry.Pid
        $wasStopped = Stop-ProcessTree -ProcessId $pidValue -ExpectedLauncherPath $entry.Launcher
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
