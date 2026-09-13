param(
    [Parameter(Mandatory=$true)][string]$Dataset,
    [Parameter(Mandatory=$true)][string]$RawOutput,
    [Parameter(Mandatory=$true)][string]$ManifestOutput
)
$ErrorActionPreference = 'Stop'
& python (Join-Path $PSScriptRoot 'prepare-rust-s5-evidence.py') --dataset $Dataset --raw-output $RawOutput --manifest-output $ManifestOutput
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
