param(
    [string]$Server = "127.0.0.1",
    [ValidateRange(1, 65535)][int]$Port = 8080,
    [string]$Nickname = "",
    [string]$Email = ""
)
$ErrorActionPreference = "Stop"
$pongRoot = Split-Path -Parent $PSScriptRoot
$pongPython = Join-Path $pongRoot '.venv\Scripts\python.exe'
if (-not (Test-Path -LiteralPath $pongPython)) {
    & python -m venv (Join-Path $pongRoot '.venv')
    if ($LASTEXITCODE -ne 0) { throw "No se pudo crear el entorno Python." }
}
$env:PYGAME_HIDE_SUPPORT_PROMPT = '1'
& $pongPython -c "import importlib.util,sys; sys.exit(1) if importlib.util.find_spec('pygame') is None else None; import pygame; sys.exit(pygame.version.ver != '2.6.1')"
if ($LASTEXITCODE -ne 0) {
    & $pongPython -m pip install -r (Join-Path $pongRoot 'pong_client\requirements.txt')
    if ($LASTEXITCODE -ne 0) { throw "No se pudo instalar Pygame." }
}
$pongClient = Join-Path $pongRoot 'pong_client\main.py'
if ($Nickname -and $Email) { & $pongPython $pongClient $Server $Port $Nickname $Email }
else { & $pongPython $pongClient $Server $Port }
exit $LASTEXITCODE
