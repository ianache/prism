param(
    [Parameter(Mandatory=$true)][string]$Dataset,
    [Parameter(Mandatory=$true)][string]$RawOutput,
    [Parameter(Mandatory=$true)][string]$ManifestOutput
)
$ErrorActionPreference = 'Stop'
$datasetPath = (Resolve-Path -LiteralPath $Dataset).Path
if (Test-Path -LiteralPath $RawOutput) { throw "raw output already exists: $RawOutput" }
$manifestPath = Join-Path $datasetPath 'manifest.json'
$digestPath = Join-Path $datasetPath 'manifest.sha256'
if (-not (Test-Path -LiteralPath $manifestPath)) { throw "missing dataset manifest" }
if (-not (Test-Path -LiteralPath $digestPath)) { throw "missing dataset digest" }
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$digest = (Get-Content -LiteralPath $digestPath -Raw).Trim()
$root = (git rev-parse --show-toplevel).Trim()
$commit = (git rev-parse HEAD).Trim()
$cargo = (& 'C:\Users\ilver\.cargo\bin\cargo.exe' --version).Trim()
$record = [ordered]@{
    repository = $root
    commit = $commit
    dataset = $datasetPath
    dataset_id = $manifest.dataset_id
    dataset_digest = $digest
    scenario = 'S5'
    protocol_version = '1.1'
    samples = 100000
    repetitions = 5
    duration_seconds = 900
    concurrency = 1
    cargo = $cargo
    os = [System.Environment]::OSVersion.VersionString
    processor_count = [System.Environment]::ProcessorCount
    output = [System.IO.Path]::GetFullPath($RawOutput)
    output_available = $true
}
$parent = Split-Path -Parent ([System.IO.Path]::GetFullPath($ManifestOutput))
New-Item -ItemType Directory -Force -Path $parent | Out-Null
$record | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $ManifestOutput -Encoding utf8
Write-Output (ConvertTo-Json $record -Compress)
