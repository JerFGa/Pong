param(
    [ValidateRange(1, 65535)][int]$Port = 8080,
    [string]$LogFile = "server.log"
)
$ErrorActionPreference = "Stop"
$pongRoot = Split-Path -Parent $PSScriptRoot
$pongLinuxPath = & wsl -d Ubuntu --exec wslpath -a $pongRoot.Replace('\', '/')
if ($LASTEXITCODE -ne 0 -or -not $pongLinuxPath) { throw "No se pudo acceder a Ubuntu/WSL." }
$pongLinuxRoot = $pongLinuxPath.Trim()
& wsl -d Ubuntu --exec bash "$pongLinuxRoot/scripts/start-server.sh" $Port $LogFile
exit $LASTEXITCODE
