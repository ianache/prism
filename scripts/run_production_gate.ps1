[CmdletBinding()]
param(
    [int]$Frames = 100000,
    [int]$BatchSize = 64,
    [ValidateSet('tcp', 'http')]
    [string]$Protocol = 'tcp',
    [string]$EvidenceDir = '',
    [string]$LinuxDistribution = ''
)

$ErrorActionPreference = 'Stop'

function Write-Failure([string]$Message) {
    [Console]::Error.WriteLine("S22_LOCAL_GATE_ERROR: $Message")
    exit 2
}

function Test-CommandAvailable([string]$Name) {
    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

if ($Frames -le 0) { Write-Failure 'Frames must be positive' }
if ($BatchSize -le 0) { Write-Failure 'BatchSize must be positive' }
if ($PSBoundParameters.Keys | Where-Object { $_ -match 'token|pem|privatekey' }) {
    Write-Failure 'Secrets such as token or private key must not be passed as arguments'
}

if (-not (Test-CommandAvailable 'wsl.exe')) { Write-Failure 'wsl.exe is required' }
if (-not (Test-CommandAvailable 'docker.exe')) { Write-Failure 'docker.exe is required' }

$root = (& git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($root)) {
    Write-Failure 'could not resolve repository root'
}

if ([string]::IsNullOrWhiteSpace($EvidenceDir)) {
    $EvidenceDir = Join-Path $root 'artifacts/s22-local'
} elseif (-not [IO.Path]::IsPathRooted($EvidenceDir)) {
    $EvidenceDir = Join-Path $root $EvidenceDir
}
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null

if ([string]::IsNullOrWhiteSpace($LinuxDistribution)) {
    $LinuxDistribution = (& wsl.exe -l -q 2>$null | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -First 1).Trim()
}
if ([string]::IsNullOrWhiteSpace($LinuxDistribution)) {
    Write-Failure 'no Linux distribution is installed; install WSL separately and retry'
}

$rootLinux = (& wsl.exe --distribution $LinuxDistribution -- wslpath -a -u $root 2>$null).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($rootLinux)) {
    Write-Failure "could not resolve repository path in distribution '$LinuxDistribution'"
}
$evidenceLinux = (& wsl.exe --distribution $LinuxDistribution -- wslpath -a -u $EvidenceDir 2>$null).Trim()
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($evidenceLinux)) {
    Write-Failure 'could not resolve evidence path in Linux'
}

$probe = & wsl.exe --distribution $LinuxDistribution -- bash -lc 'command -v bash >/dev/null && docker compose version >/dev/null'
if ($LASTEXITCODE -ne 0) {
    Write-Failure "bash and Docker Compose are required inside distribution '$LinuxDistribution'"
}

function ConvertTo-BashLiteral([string]$Value) {
    return "'" + $Value.Replace("'", "'\"'\"'") + "'"
}

$scriptLinux = ConvertTo-BashLiteral (Join-Path $rootLinux 'scripts/docker_production_readiness.sh')
$evidenceArg = ConvertTo-BashLiteral $evidenceLinux
$protocolArg = ConvertTo-BashLiteral $Protocol
$command = "export PRISM_PROTOCOL=$protocolArg PRISM_ALLOW_CYCLE=false; bash $scriptLinux --frames $Frames --batch-size $BatchSize --evidence-dir $evidenceArg"
Write-Output "S22_LOCAL_GATE: distribution=$LinuxDistribution protocol=$Protocol frames=$Frames evidence=$EvidenceDir"
& wsl.exe --distribution $LinuxDistribution -- bash -lc $command
$exitCode = $LASTEXITCODE
Write-Output "S22_LOCAL_GATE: exit_code=$exitCode evidence=$EvidenceDir"
exit $exitCode
