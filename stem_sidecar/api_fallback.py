"""
api_fallback.py — Separação de stems via Moises API.
Usado quando GPU não está disponível e o áudio é longo (> 600 s).
Requer a variável de ambiente MOISES_API_KEY.
"""
import os
import time
import pathlib
import urllib.request
import requests

MOISES_API_KEY: str = os.environ.get("MOISES_API_KEY", "")
MOISES_API_URL: str = "https://developer-api.moises.ai/api/job"

# Intervalo de polling em segundos
_POLL_INTERVAL = 5
# Máximo de tentativas: 360 × 5 s = 30 minutos
_MAX_POLLS = 360


def separate_via_api(
    audio_path: str,
    output_dir: str,
    progress_cb,
) -> list[dict]:
    """
    Separa stems usando a API do Moises como fallback.

    Args:
        audio_path:  Caminho absoluto do arquivo de áudio fonte.
        output_dir:  Diretório onde os stems serão salvos.
        progress_cb: Callable(percent: int, eta_seconds: int).

    Returns:
        Lista de {"stem_type": str, "path": str}.

    Raises:
        RuntimeError: se MOISES_API_KEY não estiver definida,
                      se o job falhar, ou se o timeout for atingido.
    """
    api_key = os.environ.get("MOISES_API_KEY", MOISES_API_KEY)
    if not api_key:
        raise RuntimeError(
            "MOISES_API_KEY não definida. "
            "Configure a variável de ambiente para usar o fallback via API."
        )

    headers = {
        "Authorization": api_key,
        "Content-Type": "application/json",
    }

    # 1. Submeter job
    payload = {
        "name": "stem-separation",
        "workflow": "moises/stem-4",
        "params": {"inputUrl": f"file://{audio_path}"},
    }
    resp = requests.post(MOISES_API_URL, json=payload, headers=headers, timeout=30)
    resp.raise_for_status()
    job_id: str = resp.json()["id"]

    # 2. Polling até concluir
    for i in range(_MAX_POLLS):
        time.sleep(_POLL_INTERVAL)

        status_resp = requests.get(
            f"{MOISES_API_URL}/{job_id}", headers=headers, timeout=30
        )
        status_resp.raise_for_status()
        data = status_resp.json()
        status: str = data.get("status", "PENDING")

        # Progresso estimado (a API não devolve percent real)
        estimated_pct = min(i * 100 // _MAX_POLLS, 95)
        estimated_eta = max(0, (_MAX_POLLS - i) * _POLL_INTERVAL)
        progress_cb(estimated_pct, estimated_eta)

        if status == "SUCCEEDED":
            # Baixar stems para output_dir
            pathlib.Path(output_dir).mkdir(parents=True, exist_ok=True)
            stems: list[dict] = []
            result: dict = data.get("result", {})
            for key, url in result.items():
                dest = str(pathlib.Path(output_dir) / f"{key}.wav")
                urllib.request.urlretrieve(url, dest)
                stems.append({"stem_type": key, "path": dest})
            return stems

        elif status in ("FAILED", "CANCELLED"):
            raise RuntimeError(
                f"Job da API Moises terminou com status '{status}' (id={job_id})"
            )

    raise RuntimeError(
        f"Timeout após {_MAX_POLLS * _POLL_INTERVAL // 60} minutos aguardando API Moises."
    )
