# setup.ps1 — Configura o ambiente Python do sidecar de separação de stems.
# Compatível com Windows, Python 3.11 ou 3.12.
#
# Uso (PowerShell como usuário normal):
#   Set-ExecutionPolicy -Scope CurrentUser RemoteSigned   # apenas uma vez
#   .\setup.ps1

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$VenvDir   = Join-Path $ScriptDir ".venv"

# Localizar Python 3.11 ou 3.12
$PythonBin = $null
$Candidates = @("python3.12", "python3.11", "python3", "python")

foreach ($candidate in $Candidates) {
    try {
        $versionRaw = & $candidate -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>$null
        if ($versionRaw -match "^3\.(11|12)$") {
            $PythonBin = $candidate
            Write-Host "Python encontrado: $PythonBin ($versionRaw)"
            break
        }
    } catch {}
}

if (-not $PythonBin) {
    Write-Error "ERRO: Python 3.11 ou 3.12 nao encontrado. Instale antes de continuar."
    exit 1
}

# Criar/reutilizar virtualenv
if (Test-Path $VenvDir) {
    Write-Host "Ambiente virtual ja existe em $VenvDir. Reutilizando."
} else {
    Write-Host "Criando ambiente virtual em $VenvDir ..."
    & $PythonBin -m venv $VenvDir
}

$PipBin    = Join-Path $VenvDir "Scripts\pip.exe"
$PythonVenv = Join-Path $VenvDir "Scripts\python.exe"

Write-Host "Atualizando pip ..."
& $PipBin install --upgrade pip --quiet

Write-Host "Instalando dependencias de $ScriptDir\requirements.txt ..."
& $PipBin install -r (Join-Path $ScriptDir "requirements.txt")

Write-Host ""
Write-Host "Configuracao concluida."
Write-Host "Para ativar o ambiente: $VenvDir\Scripts\Activate.ps1"
Write-Host "Para testar o sidecar:"
Write-Host "  echo '{\"audio_path\":\"C:/nao/existe.mp3\",\"model\":\"htdemucs_ft\",\"output_dir\":\"C:/tmp/stems\"}' | $PythonVenv $ScriptDir\main.py"
