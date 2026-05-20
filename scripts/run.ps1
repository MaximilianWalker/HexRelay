$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $root

& node (Join-Path $PSScriptRoot 'runtime/local.mjs') start @args
exit $LASTEXITCODE
