param(
    [string]$ComposeFile = "docker-compose.yml",
    [string]$EnvFile = ".env",
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"

function Invoke-Compose {
    param([string[]]$Arguments)
    & docker compose --env-file $EnvFile -f $ComposeFile @Arguments
    if ($LASTEXITCODE -ne 0) { throw "docker compose falló con código $LASTEXITCODE" }
}

try {
    $configArgs = @("config")
    Invoke-Compose $configArgs

    $upArgs = @("up", "-d")
    if (-not $NoBuild) { $upArgs += "--build" }
    Invoke-Compose $upArgs

    Invoke-Compose @("ps")
    Invoke-Compose @("run", "--rm", "client")
    if ($LASTEXITCODE -ne 0) { throw "El cliente TLS no devolvió una respuesta válida" }
    Write-Output "S14_DOCKER_TLS_OK"
}
finally {
    try { Invoke-Compose @("down") } catch { Write-Warning $_.Exception.Message }
}
