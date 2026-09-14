param(
    [string]$EnvFile = ".env",
    [int]$FrameTarget = 4,
    [string]$InputGlob = "p0-valid-*.bin",
    [int]$BatchSize = 1
)

$ErrorActionPreference = "Stop"

function Invoke-Compose {
    param([string[]]$Arguments)
    & docker compose --env-file $EnvFile --profile telemetry @Arguments
    if ($LASTEXITCODE -ne 0) { throw "docker compose falló con código $LASTEXITCODE" }
}

try {
    Invoke-Compose @("config")
    Invoke-Compose @("up", "-d", "--build", "server")
    $result = & docker compose --env-file $EnvFile --profile telemetry run --build --rm `
        -e "PRISM_FRAME_TARGET=$FrameTarget" `
        -e "PRISM_INPUT_GLOB=$InputGlob" `
        -e "PRISM_BATCH_SIZE=$BatchSize" telemetry-client
    if ($LASTEXITCODE -ne 0) { throw "El cliente de telemetría falló" }
    $result | Write-Output
    if (-not ($result -match "S15_TELEMETRY_OK")) { throw "No se encontró el marcador S15" }
    Write-Output "S15_DOCKER_TELEMETRY_OK"
}
finally {
    try { Invoke-Compose @("down") } catch { Write-Warning $_.Exception.Message }
}
