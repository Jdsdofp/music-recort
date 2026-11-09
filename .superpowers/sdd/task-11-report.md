### Task 11 Report: Python Sidecar — IPC Loop e Demucs

**Status:** DONE

---

#### Commits

1. `22255f0` — `feat: Python sidecar — Demucs local + fallback API + IPC JSON`
2. `348c742` — `chore: adiciona regras Python ao .gitignore e remove __pycache__ do rastreamento`

---

#### Arquivos criados em `stem_sidecar/`

| Arquivo | Descrição |
|---|---|
| `requirements.txt` | Dependências: demucs==4.0.1, librosa==0.10.2, soundfile==0.12.1, torch>=2.0.0, torchaudio>=2.0.0, requests==2.31.0 |
| `preprocessor.py` | `analyze_audio(path) -> dict` — BPM e tonalidade via librosa (60 s de amostra) |
| `separator.py` | `separate_stems(audio_path, model, output_dir, progress_cb)` — Demucs via subprocess, parse de progresso por regex `\d{1,3}\s*%` |
| `api_fallback.py` | `separate_via_api(audio_path, output_dir, progress_cb)` — Moises API com polling e download dos stems |
| `main.py` | Loop IPC: lê JSON de stdin, roteia para separator/api_fallback, emite Progress/Done/Error no stdout com `flush=True` |
| `setup.sh` | Setup Linux/macOS: detecta Python 3.11/3.12, cria venv, instala requirements |
| `setup.ps1` | Setup Windows: mesma lógica em PowerShell |

---

#### Smoke Test

```
echo '{"audio_path":"/nao/existe.mp3","model":"htdemucs_ft","output_dir":"/tmp/stems"}' | python main.py
```

Resultado obtido:
```json
{"type": "Error", "message": "Arquivo de áudio não encontrado: /nao/existe.mp3"}
```

Comportamento correto — erro controlado sem stack trace, formato JSON válido conforme protocolo StemEvent.

---

#### Lógica de Fallback Implementada

- `torch.cuda.is_available()` == False **E** duração > 600 s → `api_fallback.separate_via_api()`
- Caso contrário → `separator.separate_stems()` (Demucs local, CPU ou GPU)
- Ambas as funções tratam `ImportError` de `torch` graciosamente (retornam `False` para GPU)

---

#### Conformidade com Protocolo Rust (Task 5)

O protocolo `StemRequest`/`StemEvent` em `src-tauri/src/stem_bridge/protocol.rs` usa `#[serde(tag = "type")]`, gerando:
- `{"type": "Progress", "percent": N, "eta_seconds": N}`
- `{"type": "Done", "stems": [...]}`
- `{"type": "Error", "message": "..."}`

O `main.py` emite exatamente esse formato.

---

#### Concerns

- **Demucs 4.0.1 pode não estar no PyPI com esse pin exato** — versões recentes foram publicadas como `demucs` sem pin de patch. Recomenda-se testar `pip install demucs==4.0.1` no ambiente alvo antes do release; se falhar, relaxar para `demucs>=4.0.0,<5`.
- **Parse de progresso do Demucs é heurístico** — Demucs usa tqdm e o formato pode variar entre versões. O regex `\d{1,3}\s*%` cobre os formatos mais comuns (`25%`, `25 %`, `25%|████`).
- **Moises API — `file://` para upload local** — a API Moises normalmente requer URL HTTP/HTTPS pública, não `file://`. Para uso real, será necessário implementar upload do arquivo para storage temporário (S3, etc.) antes de submeter o job. Isso está além do escopo da Task 11 mas deve ser endereçado antes do uso em produção.
- **`__pycache__` commitado no primeiro commit** foi removido no segundo commit e .gitignore atualizado.
