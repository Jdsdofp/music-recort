#!/usr/bin/env bash
# setup.sh — Configura o ambiente Python do sidecar de separação de stems.
# Compatível com Linux e macOS, Python 3.11 ou 3.12.
#
# Uso:
#   chmod +x setup.sh
#   ./setup.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="${SCRIPT_DIR}/.venv"

# Verificar Python 3.11 ou 3.12
PYTHON_BIN=""
for candidate in python3.12 python3.11 python3 python; do
    if command -v "$candidate" &>/dev/null; then
        version=$("$candidate" -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>/dev/null || echo "")
        if [[ "$version" == "3.11" || "$version" == "3.12" ]]; then
            PYTHON_BIN="$candidate"
            echo "Python encontrado: $PYTHON_BIN ($version)"
            break
        fi
    fi
done

if [[ -z "$PYTHON_BIN" ]]; then
    echo "ERRO: Python 3.11 ou 3.12 não encontrado. Instale antes de continuar." >&2
    exit 1
fi

# Criar/recriar virtualenv
if [[ -d "$VENV_DIR" ]]; then
    echo "Ambiente virtual já existe em $VENV_DIR. Reutilizando."
else
    echo "Criando ambiente virtual em $VENV_DIR ..."
    "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

source "${VENV_DIR}/bin/activate"

echo "Atualizando pip ..."
pip install --upgrade pip --quiet

echo "Instalando dependências de ${SCRIPT_DIR}/requirements.txt ..."
pip install -r "${SCRIPT_DIR}/requirements.txt"

echo ""
echo "Configuração concluída."
echo "Para ativar o ambiente: source ${VENV_DIR}/bin/activate"
echo "Para testar o sidecar:"
echo "  echo '{\"audio_path\":\"/nao/existe.mp3\",\"model\":\"htdemucs_ft\",\"output_dir\":\"/tmp/stems\"}' | python ${SCRIPT_DIR}/main.py"
