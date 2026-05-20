$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $root

& node (Join-Path $PSScriptRoot 'runtime/local.mjs') status @args
exit $LASTEXITCODE
