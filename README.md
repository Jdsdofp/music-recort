# Music Recort

DAW desktop para Windows com separação de stems via IA. Construído com Rust + Tauri 2 + React + TypeScript + Python (sidecar).

## Pré-requisitos

| Ferramenta | Versão mínima | Observação |
|---|---|---|
| Rust | 1.77+ | Toolchain `x86_64-pc-windows-gnu` via MSYS2 |
| MSYS2 / MinGW-w64 | Atual | GCC em `C:/msys64/mingw64/bin` |
| Node.js | 20+ | npm 10+ incluído |
| Python | 3.11 ou 3.12 | Para o sidecar de separação de stems |
| GPU NVIDIA (opcional) | CUDA 11.8+ | Acelera o Demucs; sem GPU usa CPU ou API de fallback |

### Instalação do Rust com toolchain GNU

```powershell
# Instalar rustup (https://rustup.rs/) e adicionar o toolchain MinGW
rustup target add x86_64-pc-windows-gnu
rustup toolchain install stable-x86_64-pc-windows-gnu
```

## Configuração

### 1. Dependências Node

```powershell
npm install
```

### 2. Sidecar Python (separação de stems)

```powershell
cd stem_sidecar
.\setup.ps1
```

O script cria `.venv` e instala todas as dependências Python (demucs, torch, etc.).

> Para separação sem GPU local, defina a variável de ambiente `MOISES_API_KEY` com sua chave da API Moises.ai:
> ```powershell
> $env:MOISES_API_KEY = "sua-chave-aqui"
> ```
> O app detecta automaticamente se a GPU está disponível e, caso não esteja ou o áudio seja muito longo (>10 min), usa o fallback via API.

### 3. Verificar compilação Rust

```powershell
cd src-tauri
$env:PATH = "C:/msys64/mingw64/bin:" + $env:PATH
cargo check
```

## Como rodar

### Desenvolvimento

```powershell
# Terminal 1 — iniciar o sidecar Python manualmente (necessário para separação de stems)
cd stem_sidecar
.\.venv\Scripts\Activate.ps1
python main.py

# Terminal 2 — rodar o app Tauri
npm run tauri dev
```

### Build de produção

```powershell
npm run tauri build
```

O binário gerado fica em `src-tauri/target/release/music-recort.exe`.

## Funcionalidades

- Importação de áudio: MP3, WAV, FLAC, AAC, OGG
- Playback com controles de transporte (play, pause, seek)
- Visualização de waveform na timeline
- Gerenciamento de projetos `.mrt` (JSON + compressão zstd)
- Auto-save a cada 30 segundos
- Undo/redo (até 100 ações)
- Separação de stems via Demucs (modelos: `htdemucs`, `htdemucs_ft`, `mdx_extra`)
- Fallback automático para API Moises.ai (sem GPU ou áudio >10 min)
- Progresso de separação em tempo real (streaming via named pipe)
- Stems como faixas na timeline após separação
- Mixer com controle de volume e pan por faixa
- Detecção de BPM e tonalidade

## Estrutura do projeto

```
music_recort/
├── src/                    # Frontend React + TypeScript
│   ├── components/         # Componentes UI (Timeline, Mixer, Transport, etc.)
│   ├── hooks/              # Hooks (useProject, useStemSeparation, etc.)
│   └── store/              # Estado global Zustand
├── src-tauri/              # Backend Rust + Tauri 2
│   ├── src/
│   │   ├── audio_engine/   # Decoder, player, waveform
│   │   ├── project_manager/# Serialização .mrt, histórico undo/redo
│   │   ├── stem_bridge/    # IPC com sidecar Python
│   │   └── commands/       # Handlers Tauri (audio, project, mixer, stems)
│   └── capabilities/       # Permissões Tauri
└── stem_sidecar/           # Sidecar Python
    ├── main.py             # Entrada — lê JSON do stdin, escreve progresso
    ├── separator.py        # Integração Demucs (3 modelos)
    ├── api_fallback.py     # Fallback Moises.ai
    ├── preprocessor.py     # Detecção BPM/tonalidade
    ├── requirements.txt    # Dependências Python
    └── setup.ps1           # Script de configuração (Windows)
```

## Variáveis de ambiente

| Variável | Descrição |
|---|---|
| `MOISES_API_KEY` | Chave da API Moises.ai para fallback de separação de stems |
