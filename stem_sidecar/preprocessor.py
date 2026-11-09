"""
preprocessor.py — Análise de áudio: BPM e tonalidade.
Usado opcionalmente antes da separação de stems.
"""
import librosa


def analyze_audio(path: str) -> dict:
    """
    Detecta BPM e tonalidade do arquivo de áudio.

    Carrega apenas os primeiros 60 segundos para rapidez.
    Retorna dict com 'bpm', 'key' e, em caso de erro, 'error'.
    """
    try:
        y, sr = librosa.load(path, sr=None, mono=True, duration=60.0)
        tempo, _ = librosa.beat.beat_track(y=y, sr=sr)
        # tempo pode ser array em versões recentes do librosa
        if hasattr(tempo, '__len__'):
            tempo = float(tempo[0]) if len(tempo) > 0 else 120.0
        chroma = librosa.feature.chroma_cqt(y=y, sr=sr)
        key_idx = int(chroma.mean(axis=1).argmax())
        keys = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
        return {"bpm": float(round(float(tempo), 1)), "key": keys[key_idx]}
    except Exception as e:
        return {"bpm": 120.0, "key": "C", "error": str(e)}
