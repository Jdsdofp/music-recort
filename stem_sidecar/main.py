"""
main.py — Loop IPC do sidecar de separação de stems.

Protocolo:
  stdin  (uma linha por requisição):
      {"audio_path": "...", "model": "htdemucs_ft", "output_dir": "..."}

  stdout (um JSON por linha, flush imediato):
      {"type": "Progress", "percent": 45, "eta_seconds": 80}
      {"type": "Done",     "stems": [{"stem_type": "vocals", "path": "/tmp/vocals.wav"}, ...]}
      {"type": "Error",    "message": "..."}

Lógica de roteamento:
  - Se GPU disponível                   → Demucs local
  - Se sem GPU e duração ≤ 600 s        → Demucs local (CPU)
  - Se sem GPU e duração > 600 s        → API Moises (requer MOISES_API_KEY)
"""
from __future__ import annotations

import sys
import json
import os

# ---------------------------------------------------------------------------
# Importações opcionais: tratadas graciosamente para permitir smoke test
# sem dependências instaladas.
# ---------------------------------------------------------------------------
try:
    import torch as _torch  # noqa: F401 — verificação de GPU

    def _has_gpu() -> bool:
        return _torch.cuda.is_available()

except ImportError:
    def _has_gpu() -> bool:  # type: ignore[misc]
        return False


def _get_duration(audio_path: str) -> float | None:
    """Retorna duração em segundos usando librosa; None em caso de falha."""
    try:
        import librosa  # type: ignore
        return float(librosa.get_duration(path=audio_path))
    except Exception as e:
        print(f"Warning: could not determine audio duration: {e}", flush=True)
        return None


# ---------------------------------------------------------------------------
# Emissor de eventos
# ---------------------------------------------------------------------------

def emit(event: dict) -> None:
    """Serializa e escreve um evento JSON no stdout com flush imediato."""
    print(json.dumps(event), flush=True)


# ---------------------------------------------------------------------------
# Roteador de requisições
# ---------------------------------------------------------------------------

def handle_request(req: dict) -> None:
    """Processa um StemRequest e emite StemEvents para stdout."""
    audio_path: str = req.get("audio_path", "")
    model: str = req.get("model", "htdemucs_ft")
    output_dir: str = req.get("output_dir", "")

    if not audio_path:
        emit({"type": "Error", "message": "Campo 'audio_path' ausente na requisição."})
        return
    if not output_dir:
        emit({"type": "Error", "message": "Campo 'output_dir' ausente na requisição."})
        return

    def progress_cb(percent: int, eta_seconds: int) -> None:
        emit({"type": "Progress", "percent": percent, "eta_seconds": eta_seconds})

    # Decisão de roteamento: local vs API
    gpu_available = _has_gpu()
    use_api = False

    if not gpu_available:
        duration = _get_duration(audio_path)
        if duration is None or duration > 600:
            use_api = True

    try:
        if use_api:
            from api_fallback import separate_via_api  # type: ignore
            stems = separate_via_api(audio_path, output_dir, progress_cb)
        else:
            from separator import separate_stems  # type: ignore
            stems = separate_stems(audio_path, model, output_dir, progress_cb)

        emit({"type": "Done", "stems": stems})

    except Exception as exc:
        emit({"type": "Error", "message": str(exc)})


# ---------------------------------------------------------------------------
# Loop IPC principal
# ---------------------------------------------------------------------------

def main() -> None:
    """
    Lê linhas JSON de stdin em loop contínuo.
    Cada linha representa um StemRequest completo.
    Responde com StemEvents no stdout (um JSON por linha).
    """
    for raw_line in sys.stdin:
        line = raw_line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError as exc:
            emit({"type": "Error", "message": f"JSON inválido na entrada: {exc}"})
            continue
        handle_request(req)


if __name__ == "__main__":
    main()
