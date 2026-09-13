$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot ".."))
$runner = Join-Path $root "target\release\prism-run.exe"
if (-not (Test-Path -LiteralPath $runner)) {
    throw "No existe $runner. Ejecuta primero: cargo test --workspace --release"
}

$temp = Join-Path ([IO.Path]::GetTempPath()) ("prism-s11-manual-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
$tokenPath = Join-Path $temp "token.txt"
$stderrPath = Join-Path $temp "stderr.txt"
$token = "manual-secret"
[IO.File]::WriteAllText($tokenPath, $token + [Environment]::NewLine)

$process = Start-Process -FilePath $runner -ArgumentList @(
    "--route", "b0", "--listen", "127.0.0.1:0", "--auth-token-file", $tokenPath
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
    $address = $ready.Line.Substring(6).Trim()
    $parts = $address.Split(':')

    $client = [Net.Sockets.TcpClient]::new($parts[0], [int]$parts[1])
    $stream = $client.GetStream()
    $writer = [IO.StreamWriter]::new($stream)
    $reader = [IO.StreamReader]::new($stream)
    $writer.AutoFlush = $true

    $writer.WriteLine('{"request_id":"bad","payload_hex":"00"}')
    $bad = $reader.ReadLine() | ConvertFrom-Json
    if ($bad.error.code -ne 'AUTHENTICATION_FAILED') { throw "Se esperaba AUTHENTICATION_FAILED" }

    $writer.WriteLine('{"request_id":"good","payload_hex":"00","auth_token":"manual-secret"}')
    $good = $reader.ReadLine() | ConvertFrom-Json
    if (-not $good.ok) { throw "La solicitud autenticada fue rechazada" }

    Write-Output "MANUAL_S11_AUTH_OK unauthorized=$($bad.error.code) authorized=$($good.ok)"
    $reader.Dispose()
    $writer.Dispose()
    $client.Dispose()
}
finally {
    if ($process -and -not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
