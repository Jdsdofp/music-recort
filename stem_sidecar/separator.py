"""
separator.py — Separação de stems usando Demucs via subprocess.
Nunca importa demucs diretamente; invoca `python -m demucs` para capturar progresso.
"""
import subprocess
import sys
import re
import threading
from pathlib import Path

MODELS = ["htdemucs", "htdemucs_ft", "htdemucs_6s"]

STEM_NAMES: dict[str, list[str]] = {
    "htdemucs":    ["vocals", "drums", "bass", "other"],
    "htdemucs_ft": ["vocals", "drums", "bass", "other"],
    "htdemucs_6s": ["vocals", "drums", "bass", "other", "guitar", "piano"],
}

# Padrão de progresso: Demucs imprime linhas como "10%|..." ou "Separating track: 25%"
_PCT_RE = re.compile(r"(\d{1,3})\s*%")


def has_gpu() -> bool:
    """Retorna True se CUDA estiver disponível."""
    try:
        import torch
        return torch.cuda.is_available()
    except ImportError:
        return False


def separate_stems(
    audio_path: str,
    model: str,
    output_dir: str,
    progress_cb,
) -> list[dict]:
    """
    Separa stems usando Demucs local via subprocess.

    Args:
        audio_path:  Caminho absoluto do arquivo de áudio fonte.
        model:       Nome do modelo Demucs (htdemucs, htdemucs_ft, htdemucs_6s).
        output_dir:  Diretório raiz onde os stems serão gravados.
        progress_cb: Callable(percent: int, eta_seconds: int) chamado a cada progresso.

    Returns:
        Lista de {"stem_type": str, "path": str}.

    Raises:
        FileNotFoundError: se audio_path não existir.
        RuntimeError:      se Demucs retornar código de saída diferente de zero.
    """
    audio_file = Path(audio_path)
    if not audio_file.exists():
        raise FileNotFoundError(f"Arquivo de áudio não encontrado: {audio_path}")

    if model not in MODELS:
        model = "htdemucs_ft"

    Path(output_dir).mkdir(parents=True, exist_ok=True)

    device = "cuda" if has_gpu() else "cpu"

    cmd = [
        sys.executable, "-m", "demucs",
        "--name", model,
        "--device", device,
        "--out", output_dir,
        str(audio_file),
    ]

    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,   # stderr separado para evitar deadlock no Windows
        text=True,
        bufsize=1,
        encoding="utf-8",
        errors="replace",
    )

    last_pct = 0

    def _read_output(proc: subprocess.Popen) -> None:
        """Lê stdout E stderr em thread dedicada para evitar deadlock de buffer."""
        nonlocal last_pct

        import select as _select
        import io

        # Leitura intercalada de stdout e stderr usando threads internas
        def _drain(stream: io.TextIOBase, track_progress: bool) -> None:
            nonlocal last_pct
            for raw_line in stream:
                line = raw_line.rstrip()
                if track_progress:
                    match = _PCT_RE.search(line)
                    if match:
                        try:
                            pct = int(match.group(1))
                            if pct != last_pct:
                                last_pct = pct
                                # ETA linear simples: cada ponto percentual ≈ 1,5 s
                                eta = max(0, int((100 - pct) * 1.5))
                                progress_cb(pct, eta)
                        except ValueError:
                            pass

        # Thread para stderr (Demucs escreve progresso no stderr)
        stderr_thread = threading.Thread(
            target=_drain, args=(proc.stderr, True), daemon=True  # type: ignore[arg-type]
        )
        # Thread para stdout
        stdout_thread = threading.Thread(
            target=_drain, args=(proc.stdout, False), daemon=True  # type: ignore[arg-type]
        )
        stderr_thread.start()
        stdout_thread.start()
        stderr_thread.join()
        stdout_thread.join()
        proc.wait()

    output_thread = threading.Thread(target=_read_output, args=(process,), daemon=True)
    output_thread.start()
    output_thread.join(timeout=300)  # 5 minutos

    if output_thread.is_alive():
        process.kill()
        raise TimeoutError("Demucs timeout after 300s")

    if process.returncode != 0:
        raise RuntimeError(
            f"Demucs falhou com código de saída {process.returncode} "
            f"(modelo={model}, arquivo={audio_path})"
        )

    # Demucs grava em <output_dir>/<model>/<audio_stem>/
    audio_stem = audio_file.stem
    stems_dir = Path(output_dir) / model / audio_stem

    stems: list[dict] = []
    for stem_type in STEM_NAMES.get(model, []):
        stem_path = stems_dir / f"{stem_type}.wav"
        if stem_path.exists():
            stems.append({"stem_type": stem_type, "path": str(stem_path)})

    if not stems:
        raise RuntimeError(
            f"Nenhum stem encontrado em '{stems_dir}'. "
            "Verifique se o Demucs foi instalado corretamente."
        )

    return stems
