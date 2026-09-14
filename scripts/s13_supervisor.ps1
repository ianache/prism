param(
    [Parameter(Mandatory = $true)][ValidateSet("start", "check", "stop")][string]$Action,
    [Parameter(Mandatory = $true)][string]$StateDirectory,
    [string]$Runner,
    [string]$Certificate,
    [string]$PrivateKey,
    [string]$TokenFile,
    [int]$DrainTimeoutMs = 5000
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot ".."))
if ([string]::IsNullOrWhiteSpace($Runner)) { $Runner = Join-Path $root "target\release\prism-run.exe" }
$stateFile = Join-Path $StateDirectory "state.json"

function Read-State {
    if (-not (Test-Path -LiteralPath $stateFile)) { throw "No existe estado del supervisor" }
    Get-Content -LiteralPath $stateFile -Raw | ConvertFrom-Json
}

function Get-RunnerProcess($state) {
    $process = Get-Process -Id ([int]$state.pid) -ErrorAction SilentlyContinue
    if ($null -eq $process -or $process.ProcessName -notlike "prism-run*") {
        throw "estado stale: el PID no corresponde a prism-run"
    }
    $process
}

function Open-SecureClient($state) {
    $hostName, $portText = $state.address.Split(':')
    $tcp = [Net.Sockets.TcpClient]::new($hostName, [int]$portText)
    $ssl = [Net.Security.SslStream]::new($tcp.GetStream(), $false, { return $true })
    $ssl.AuthenticateAsClient("localhost")
    @{ Tcp = $tcp; Ssl = $ssl; Reader = [IO.StreamReader]::new($ssl); Writer = [IO.StreamWriter]::new($ssl) }
}

if ($Action -eq "start") {
    if (-not (Test-Path -LiteralPath $Runner)) { throw "No existe Runner: $Runner" }
    foreach ($required in @($Certificate, $PrivateKey, $TokenFile)) {
        if ([string]::IsNullOrWhiteSpace($required) -or -not (Test-Path -LiteralPath $required)) {
            throw "Falta un archivo TLS/token requerido"
        }
    }
    if (Test-Path -LiteralPath $stateFile) { throw "Ya existe estado del supervisor" }
    New-Item -ItemType Directory -Path $StateDirectory -Force | Out-Null
    $stderrPath = Join-Path $StateDirectory "stderr.log"
    $sentinel = Join-Path $StateDirectory "shutdown.flag"
    $process = Start-Process -FilePath $Runner -ArgumentList @(
        "--route", "b2", "--listen", "127.0.0.1:0", "--tls-cert", $Certificate,
        "--tls-key", $PrivateKey, "--auth-token-file", $TokenFile,
        "--shutdown-file", $sentinel, "--drain-timeout-ms", $DrainTimeoutMs
    ) -RedirectStandardError $stderrPath -WindowStyle Hidden -PassThru
    try {
        $address = $null
        for ($attempt = 0; $attempt -lt 100 -and $null -eq $address; $attempt++) {
            Start-Sleep -Milliseconds 100
            if (Test-Path -LiteralPath $stderrPath) {
                $match = Select-String -Path $stderrPath -Pattern '^READY (.+)$' | Select-Object -First 1
                if ($match) { $address = $match.Matches[0].Groups[1].Value.Trim() }
            }
        }
        if ($null -eq $address) { throw "El runner no anunció READY" }
        [pscustomobject]@{ pid = $process.Id; address = $address; sentinel = $sentinel; stderr = $stderrPath } |
            ConvertTo-Json | Set-Content -LiteralPath $stateFile -Encoding utf8
        Write-Output "S13_START_OK address=$address pid=$($process.Id)"
    }
    catch {
        if (-not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
        throw
    }
    exit 0
}

$state = Read-State
$process = Get-RunnerProcess $state

if ($Action -eq "check") {
    if ([string]::IsNullOrWhiteSpace($TokenFile) -or -not (Test-Path -LiteralPath $TokenFile)) { throw "Falta TokenFile" }
    $token = (Get-Content -LiteralPath $TokenFile -Raw).Trim()
    if ([string]::IsNullOrWhiteSpace($token)) { throw "Token vacío" }
    $client = Open-SecureClient $state
    try {
        $client.Writer.AutoFlush = $true
        $client.Writer.WriteLine('{"request_id":"s13-check","payload_hex":"00","auth_token":"' + $token + '"}')
        $response = $client.Reader.ReadLine() | ConvertFrom-Json
        if (-not $response.ok -or $response.route -ne "b2") { throw "health check rechazado" }
        Write-Output "S13_CHECK_OK route=$($response.route) ok=$($response.ok)"
    }
    finally { $client.Reader.Dispose(); $client.Writer.Dispose(); $client.Ssl.Dispose(); $client.Tcp.Dispose() }
    exit 0
}

if (-not (Test-Path -LiteralPath $state.sentinel)) { New-Item -ItemType File -Path $state.sentinel | Out-Null }
$deadline = [DateTime]::UtcNow.AddMilliseconds($DrainTimeoutMs + 2000)
while (-not $process.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 100 }
if (-not $process.HasExited) { throw "STOPPED no fue observado dentro del timeout" }
if (-not (Select-String -Path $state.stderr -Pattern '^STOPPED$')) { throw "El proceso terminó sin estado STOPPED" }
Remove-Item -LiteralPath $stateFile -Force
Write-Output "S13_STOP_OK"
