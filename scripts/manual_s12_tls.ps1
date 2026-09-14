param(
    [Parameter(Mandatory = $true)][string]$Certificate,
    [Parameter(Mandatory = $true)][string]$PrivateKey,
    [Parameter(Mandatory = $true)][string]$TokenFile
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot ".."))
$runner = Join-Path $root "target\release\prism-run.exe"
if (-not (Test-Path -LiteralPath $runner)) { throw "Falta $runner" }

$temp = Join-Path ([IO.Path]::GetTempPath()) ("prism-s12-manual-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
$stderrPath = Join-Path $temp "stderr.txt"
$process = Start-Process -FilePath $runner -ArgumentList @(
    "--route", "b2", "--listen", "127.0.0.1:0", "--tls-cert", $Certificate,
    "--tls-key", $PrivateKey, "--auth-token-file", $TokenFile
) -WorkingDirectory $root -RedirectStandardError $stderrPath -PassThru

try {
    $ready = $null
    for ($attempt = 0; $attempt -lt 50 -and $null -eq $ready; $attempt++) {
        Start-Sleep -Milliseconds 100
        if (Test-Path -LiteralPath $stderrPath) {
            $ready = Select-String -Path $stderrPath -Pattern '^READY ' | Select-Object -First 1
        }
    }
    if ($null -eq $ready) { throw "El runner no anunció READY" }
    $hostName, $portText = $ready.Line.Substring(6).Trim().Split(':')
    $tcp = [Net.Sockets.TcpClient]::new($hostName, [int]$portText)
    # Para certificados locales de prueba; use una CA confiable para entornos reales.
    $ssl = [Net.Security.SslStream]::new($tcp.GetStream(), $false, { return $true })
    $ssl.AuthenticateAsClient("localhost")
    $writer = [IO.StreamWriter]::new($ssl)
    $reader = [IO.StreamReader]::new($ssl)
    $writer.AutoFlush = $true
    $writer.WriteLine('{"request_id":"manual","payload_hex":"00","auth_token":"' + (Get-Content -Raw $TokenFile).Trim() + '"}')
    $response = $reader.ReadLine() | ConvertFrom-Json
    if (-not $response.ok) { throw "La solicitud TLS autenticada fue rechazada" }
    Write-Output "MANUAL_S12_TLS_OK route=$($response.route) ok=$($response.ok)"
    $reader.Dispose(); $writer.Dispose(); $ssl.Dispose(); $tcp.Dispose()
}
finally {
    if ($process -and -not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
