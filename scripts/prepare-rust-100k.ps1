param(
    [string]$Output = 'D:\02-PERSONAL\TOOLS\prism-datasets\p0-100k'
)

$seed = '0x505249534D5F5631'
python -m tools.dataset.generate --output $Output --seed $seed --count 100000
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
python -m tools.dataset.verify --dataset $Output
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$digest = (Get-Content -LiteralPath (Join-Path $Output 'manifest.sha256') -Raw).Trim()
Write-Output "dataset=$Output"
Write-Output "count=100000"
Write-Output "manifest_sha256=$digest"
