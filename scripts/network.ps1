param(
    [string]$Profile = 'normal',
    [string]$Target = '',
    [switch]$Reset,
    [switch]$Json,
    [switch]$Force,
    [Alias('h')]
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

if ($Help) {
    Write-Host 'Usage: network.ps1 [-Profile normal|offline-alice|partition-alice-bob|path] [-Target instance-id|container] [-Reset] [-Json] [-Force]'
    exit 0
}

$root = Split-Path -Parent $PSScriptRoot
$argsList = @((Join-Path $PSScriptRoot 'network.mjs'))
if ($Reset) {
    $argsList += '--reset'
} else {
    $argsList += @('--profile', $Profile)
}
if ($Target.Trim()) {
    $argsList += @('--target', $Target)
}
if ($Json) {
    $argsList += '--json'
}
if ($Force) {
    $argsList += '--force'
}

node @argsList
exit $LASTEXITCODE
