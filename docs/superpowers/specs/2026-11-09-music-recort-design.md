# Music Recort — Design Spec (Fases 1 e 2)

**Data:** 2026-11-09  
**Autor:** Jaderson Moreira  
**Status:** Aprovado

---

## Visão Geral

Sistema de produção musical desktop para produtores profissionais que combina:
- **Separação de stems com IA** (vocal, bateria, baixo, melodia, guitarra, piano)
- **DAW completa** com timeline multitrack, piano roll, sequenciador de beats, mixer e efeitos

Plataforma alvo: Windows (primário), Mac (secundário).  
Stack: Rust + Tauri + React/TypeScript + Python Sidecar.

---

## Escopo — Fases 1 e 2

### Fase 1 — Motor de Áudio + UI Base
- Importação e decodificação de áudio (MP3, WAV, FLAC, AAC, OGG)
- Playback com controles de transporte (play, pause, stop, seek)
- Renderização de waveform
- Gerenciamento de projetos (.mrt)
- Auto-save e histórico de undo/redo

### Fase 2 — Separação de Stems com IA
- Demucs local (htdemucs, htdemucs_ft, htdemucs_6s)
- Fallback automático para API externa
- Progress streaming em tempo real
- Stems carregados como faixas independentes na timeline

---

## Arquitetura

```
┌─────────────────────────────────────────────────────┐
│                   TAURI (Shell)                      │
│  ┌─────────────────────┐  ┌────────────────────────┐ │
│  │   Frontend (React)  │  │   Rust Core            │ │
│  │  - Waveform viewer  │  │  - Motor de áudio      │ │
│  │  - Timeline UI      │  │    (cpal)              │ │
│  │  - Mixer UI         │  │  - Gerenciador de      │ │
│  │  - Piano Roll UI    │  │    projeto (.mrt)      │ │
│  │  - Stem viewer      │  │  - IPC com Python      │ │
│  └────────┬────────────┘  └──────────┬─────────────┘ │
│           │   Tauri Commands/Events  │               │
└───────────┼──────────────────────────┼───────────────┘
            │                          │
            │              ┌───────────▼──────────────┐
            │              │  Python Sidecar           │
            │              │  - Demucs (local GPU/CPU) │
            │              │  - Fallback: API REST     │
            │              │  - Pré-processamento FFT  │
            │              └──────────────────────────┘
```

---

## Componentes

### Rust Core (`src-tauri/src/`)

#### `audio_engine`
- **I/O de áudio:** `cpal` — cross-platform, latência profissional
- **Decodificação:** `symphonia` — MP3, WAV, FLAC, AAC, OGG
- **Processamento:** `rubberband` via FFI — pitch shift e time stretch em tempo real
- **Graph de áudio:** nós de fonte → efeitos → mixer → saída

#### `project_manager`
- Formato `.mrt`: JSON compactado com `zstd`
- Contém: metadados, BPM, tonalidade, referências a stems, configurações de faixas, automações
- Auto-save a cada 30 segundos
- Histórico de undo/redo até 100 ações

#### `stem_bridge`
- Comunicação via named pipes (Windows) / Unix sockets (Mac)
- Protocolo: mensagens JSON com chunks de progresso (0–100%)
- Timeout: 5 minutos; fallback automático para API se Demucs falhar

#### Tauri Commands expostos
```
audio::load_file(path) → AudioInfo
audio::play()
audio::pause()
audio::seek(position_ms)
stems::separate(path, model) → EventStream<Progress>
project::save(path)
project::load(path) → Project
project::export_wav(path, settings)
mixer::set_volume(track_id, volume)
mixer::set_pan(track_id, pan)
```

---

### Frontend (`src/`)

```
src/
├── components/
│   ├── Waveform/        # Canvas WebGL — renderiza áudio com zoom
│   ├── Timeline/        # Faixas multitrack, clips arrastáveis
│   ├── Mixer/           # Faders, pan, mute/solo, sends
│   ├── PianoRoll/       # Grade MIDI, notas editáveis
│   ├── BeatSequencer/   # Grade 16-steps estilo FL Studio
│   ├── StemPanel/       # Resultado da separação, controles por stem
│   └── Transport/       # Play/pause/stop, BPM, tonalidade, posição
├── stores/              # Zustand — estado global
├── hooks/               # useAudioEngine, useStemSeparation, useProject
└── lib/
    └── tauri.ts         # Wrappers tipados dos Tauri Commands
```

**Stack:** React 19 + TypeScript + Tailwind CSS + shadcn/ui  
**Waveform:** WebGL via canvas nativo  
**Estado global:** Zustand

---

### Python Sidecar (`stem_sidecar/`)

```
stem_sidecar/
├── main.py              # Loop IPC — recebe comandos, responde com progresso
├── separator.py         # Demucs local
├── api_fallback.py      # Cliente HTTP para API externa
├── preprocessor.py      # Normalização, detecção de BPM/tonalidade (librosa)
└── models/              # Pesos do Demucs (baixados no primeiro uso, ~500MB)
```

#### Modelos Demucs

| Modelo | Stems | Qualidade | Velocidade |
|--------|-------|-----------|------------|
| `htdemucs` | 4 (vocal, drums, bass, other) | Alta | ~2× |
| `htdemucs_ft` | 4 (fine-tuned) | Máxima | ~3× |
| `htdemucs_6s` | 6 (+guitar, piano) | Máxima | ~4× |

Fallback para API ativado quando: sem GPU disponível E arquivo > 10 minutos.

---

## Fluxo de Dados — Separação de Stems

```
Usuário importa MP3
       │
       ▼
Rust: symphonia decodifica → valida formato
       │
       ▼
Tauri Command: stems::separate(path, model)
       │
       ▼
stem_bridge envia JSON via named pipe → Python sidecar
       │
       ├─► GPU disponível? → Demucs local (htdemucs_ft)
       └─► Sem GPU / arquivo longo? → API fallback
       │
       ▼
Progresso streamed: { percent: 45, eta: "1m20s" }
       │
       ▼
Stems retornam como WAVs em pasta temp do projeto
       │
       ▼
Rust carrega stems na timeline como faixas independentes
       │
       ▼
Frontend renderiza waveforms + controles de mixer por stem
```

---

## Tratamento de Erros

| Situação | Comportamento |
|----------|--------------|
| Arquivo corrompido | Erro inline na UI, log detalhado |
| Demucs sem GPU | Aviso + fallback automático para CPU |
| API offline | Notificação + opção de fila local |
| Projeto corrompido | Restaura último auto-save |
| IPC timeout (>5min) | Cancela, mostra erro com opção de retry |

---

## Estrutura de Pastas

```
music-recort/
├── src-tauri/              # Rust — motor, IPC, comandos Tauri
│   ├── src/
│   │   ├── audio_engine/
│   │   ├── project_manager/
│   │   ├── stem_bridge/
│   │   └── main.rs
│   └── Cargo.toml
├── src/                    # React frontend
│   ├── components/
│   ├── stores/
│   ├── hooks/
│   └── lib/
├── stem_sidecar/           # Python — IA e separação
│   ├── main.py
│   ├── separator.py
│   ├── api_fallback.py
│   └── requirements.txt
├── docs/
│   └── superpowers/specs/
└── package.json
```

---

## Fases Futuras (fora deste spec)

- **Fase 3:** Timeline multitrack completa + mixer com EQ, compressor, reverb
- **Fase 4:** Piano roll + suporte MIDI completo
- **Fase 5:** Sequenciador de beats (16-steps, estilo FL Studio)
- **Fase 6:** VST plugins + efeitos avançados + exportação profissional

---

## Dependências Principais

| Componente | Dependência | Versão |
|------------|-------------|--------|
| Rust audio I/O | cpal | 0.15+ |
| Rust decodificação | symphonia | 0.5+ |
| Rust time stretch | rubberband | via FFI |
| Rust compressão | zstd | 0.13+ |
| Python IA | demucs | 4.0+ |
| Python áudio | librosa | 0.10+ |
| Frontend framework | React | 19+ |
| Frontend estado | Zustand | 5+ |
| Frontend UI | shadcn/ui + Tailwind | latest |
| Desktop shell | Tauri | 2+ |
