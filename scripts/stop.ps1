$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $root

& node (Join-Path $PSScriptRoot 'runtime/local.mjs') stop @args
exit $LASTEXITCODE
