# Music Recort — Plano de Implementação (Fases 1 e 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Construir a fundação do Music Recort — motor de áudio profissional com playback, gerenciamento de projetos, UI React e separação de stems via IA (Demucs local + fallback API).

**Architecture:** Tauri 2 como shell desktop; Rust Core para motor de áudio (cpal), decodificação (symphonia) e IPC; Python Sidecar para Demucs e fallback API; React 19 + Zustand para o frontend.

**Tech Stack:** Rust 1.75+, Tauri 2, React 19, TypeScript 5, Zustand 5, Tailwind CSS, shadcn/ui, Python 3.11+, Demucs 4, librosa 0.10+, cpal 0.15+, symphonia 0.5+, zstd 0.13+.

## Global Constraints

- Rust edition: 2021
- Tauri: versão 2.x (NÃO Tauri 1.x — APIs são incompatíveis)
- Python: 3.11 ou 3.12 (Demucs não suporta 3.13 ainda)
- Formato de projeto: `.mrt` (JSON compactado com zstd)
- IPC: named pipes no Windows (`\\.\pipe\music-recort`), Unix sockets no Mac/Linux
- Stems salvos como WAV 44100Hz 32-bit float em `<projeto>/stems/`
- Auto-save a cada 30 segundos
- Histórico de undo/redo: máximo 100 ações
- Fallback para API ativado quando: sem GPU disponível E duração > 10 minutos

---

## Mapa de Arquivos

```
music-recort/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── audio_engine/
│       │   ├── mod.rs          # Exports públicos do módulo
│       │   ├── decoder.rs      # symphonia — decodifica MP3/WAV/FLAC/AAC/OGG
│       │   ├── player.rs       # cpal — playback com seek e controle de posição
│       │   └── graph.rs        # AudioGraph — nós fonte → mixer → saída
│       ├── project_manager/
│       │   ├── mod.rs
│       │   ├── types.rs        # Project, Track, Clip, MixerSettings (serde)
│       │   ├── serializer.rs   # Salva/carrega .mrt (JSON + zstd)
│       │   └── history.rs      # UndoRedoHistory<T>
│       ├── stem_bridge/
│       │   ├── mod.rs
│       │   ├── ipc.rs          # Named pipe / Unix socket — envia/recebe JSON
│       │   └── protocol.rs     # StemRequest, StemResponse, ProgressEvent (serde)
│       └── commands/
│           ├── audio.rs        # #[tauri::command] load_file, play, pause, seek
│           ├── project.rs      # #[tauri::command] save, load, export_wav
│           ├── stems.rs        # #[tauri::command] separate (emit events)
│           └── mixer.rs        # #[tauri::command] set_volume, set_pan
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── lib/
│   │   └── tauri.ts            # invoke() wrappers tipados
│   ├── stores/
│   │   ├── audioStore.ts       # posição, estado playback, AudioInfo
│   │   ├── projectStore.ts     # projeto atual, tracks, dirty flag
│   │   └── stemStore.ts        # stems, progresso de separação
│   ├── hooks/
│   │   ├── useAudioEngine.ts   # play/pause/seek via tauri.ts
│   │   ├── useProject.ts       # save/load/auto-save
│   │   └── useStemSeparation.ts # separate + escuta eventos de progresso
│   └── components/
│       ├── Transport/
│       │   └── Transport.tsx   # Play/pause/stop, BPM, posição em mm:ss
│       ├── Waveform/
│       │   └── Waveform.tsx    # Canvas 2D — renderiza PCM samples
│       ├── Timeline/
│       │   └── Timeline.tsx    # Lista de faixas com waveforms
│       ├── StemPanel/
│       │   └── StemPanel.tsx   # Progress bar + lista de stems gerados
│       └── Mixer/
│           └── Mixer.tsx       # Faders de volume e pan por faixa
├── stem_sidecar/
│   ├── requirements.txt
│   ├── main.py                 # Loop IPC — recebe StemRequest, emite Progress
│   ├── separator.py            # Demucs local (htdemucs_ft por padrão)
│   ├── api_fallback.py         # HTTP client para API externa
│   └── preprocessor.py        # Detecção de BPM/tonalidade (librosa)
└── package.json
```

---

### Task 1: Scaffolding do Projeto (Tauri + React + TypeScript)

**Files:**
- Create: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src/main.tsx`, `src/App.tsx`

**Interfaces:**
- Produces: projeto Tauri funcional que abre uma janela em branco

- [ ] **Step 1: Instalar pré-requisitos**

```bash
# Verificar Rust
rustc --version   # deve ser >= 1.75

# Instalar Tauri CLI
cargo install tauri-cli --version "^2.0"

# Verificar Node
node --version    # deve ser >= 20

# Instalar create-tauri-app
npm create tauri-app@latest
```

Quando solicitado:
- Project name: `music-recort`
- Identifier: `com.musicrecort.app`
- Frontend: `React` → `TypeScript`

- [ ] **Step 2: Entrar no diretório e instalar dependências do frontend**

```bash
cd music-recort
npm install
npm install zustand@5 @tauri-apps/api@2
npm install -D tailwindcss@3 postcss autoprefixer
npx tailwindcss init -p
```

- [ ] **Step 3: Configurar Tailwind em `src/index.css`**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  background-color: #1a1a2e;
  color: #e0e0e0;
  font-family: 'Inter', sans-serif;
  margin: 0;
  overflow: hidden;
}
```

- [ ] **Step 4: Configurar `tailwind.config.js`**

```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: { extend: {} },
  plugins: [],
}
```

- [ ] **Step 5: Substituir `src/App.tsx` pelo shell inicial**

```tsx
export default function App() {
  return (
    <div className="flex flex-col h-screen bg-[#1a1a2e] text-gray-100">
      <div className="flex-1 flex items-center justify-center">
        <p className="text-2xl font-bold text-purple-400">Music Recort</p>
      </div>
    </div>
  )
}
```

- [ ] **Step 6: Rodar o app e verificar que a janela abre**

```bash
cargo tauri dev
```

Esperado: janela desktop abre mostrando "Music Recort" em roxo.

- [ ] **Step 7: Commit**

```bash
git init
git add .
git commit -m "feat: scaffolding inicial Tauri 2 + React 19 + TypeScript"
```

---

### Task 2: Decodificador de Áudio (Rust — `audio_engine/decoder.rs`)

**Files:**
- Create: `src-tauri/src/audio_engine/mod.rs`
- Create: `src-tauri/src/audio_engine/decoder.rs`
- Modify: `src-tauri/Cargo.toml` (adicionar symphonia)

**Interfaces:**
- Produces: `decode_audio(path: &str) -> Result<AudioBuffer, AudioError>`
  - `AudioBuffer { samples: Vec<f32>, sample_rate: u32, channels: u16, duration_ms: u64 }`

- [ ] **Step 1: Adicionar symphonia ao `Cargo.toml`**

```toml
[dependencies]
symphonia = { version = "0.5", features = ["mp3", "aac", "flac", "ogg", "wav"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
```

- [ ] **Step 2: Criar `src-tauri/src/audio_engine/mod.rs`**

```rust
pub mod decoder;
pub mod player;
pub mod graph;

pub use decoder::{AudioBuffer, AudioError, decode_audio};
```

- [ ] **Step 3: Escrever o teste primeiro em `src-tauri/src/audio_engine/decoder.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_decode_returns_error_for_missing_file() {
        let result = decode_audio("/nao/existe.mp3");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AudioError::FileNotFound(_)));
    }

    #[test]
    fn test_audio_buffer_has_valid_fields() {
        // Cria WAV de teste sintético (1 segundo, 44100Hz, mono, silêncio)
        let path = create_test_wav();
        let buf = decode_audio(&path).expect("deve decodificar WAV de teste");
        assert_eq!(buf.sample_rate, 44100);
        assert_eq!(buf.channels, 1);
        assert!(buf.duration_ms >= 900 && buf.duration_ms <= 1100);
        assert!(!buf.samples.is_empty());
        std::fs::remove_file(&path).ok();
    }

    fn create_test_wav() -> String {
        use std::io::Write;
        let path = std::env::temp_dir().join("test_audio.wav");
        // WAV header mínimo: 44 bytes + 44100 amostras de silêncio (f32 LE)
        let mut f = std::fs::File::create(&path).unwrap();
        let num_samples: u32 = 44100;
        let data_size = num_samples * 4;
        let file_size = 36 + data_size;
        f.write_all(b"RIFF").unwrap();
        f.write_all(&file_size.to_le_bytes()).unwrap();
        f.write_all(b"WAVEfmt ").unwrap();
        f.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
        f.write_all(&3u16.to_le_bytes()).unwrap();  // PCM float
        f.write_all(&1u16.to_le_bytes()).unwrap();  // mono
        f.write_all(&44100u32.to_le_bytes()).unwrap();
        f.write_all(&(44100u32 * 4).to_le_bytes()).unwrap();
        f.write_all(&4u16.to_le_bytes()).unwrap();  // block align
        f.write_all(&32u16.to_le_bytes()).unwrap(); // bits per sample
        f.write_all(b"data").unwrap();
        f.write_all(&data_size.to_le_bytes()).unwrap();
        for _ in 0..num_samples {
            f.write_all(&0.0f32.to_le_bytes()).unwrap();
        }
        path.to_str().unwrap().to_string()
    }
}
```

- [ ] **Step 4: Rodar para verificar que falha**

```bash
cd src-tauri
cargo test audio_engine::decoder
```

Esperado: FAIL com `error[E0433]: failed to resolve: use of undeclared module`

- [ ] **Step 5: Implementar `decode_audio` em `decoder.rs`**

```rust
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("arquivo não encontrado: {0}")]
    FileNotFound(String),
    #[error("formato não suportado: {0}")]
    UnsupportedFormat(String),
    #[error("erro de decodificação: {0}")]
    DecodingError(String),
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
}

pub fn decode_audio(path: &str) -> Result<AudioBuffer, AudioError> {
    let file = std::fs::File::open(path)
        .map_err(|_| AudioError::FileNotFound(path.to_string()))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| AudioError::UnsupportedFormat("nenhuma faixa de áudio".to_string()))?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(_)) | Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(AudioError::DecodingError(e.to_string())),
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let spec = *audio_buf.spec();
                let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.capacity() as u64, spec);
                sample_buf.copy_interleaved_ref(audio_buf);
                all_samples.extend_from_slice(sample_buf.samples());
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(AudioError::DecodingError(e.to_string())),
        }
    }

    let total_frames = all_samples.len() as u64 / channels as u64;
    let duration_ms = (total_frames * 1000) / sample_rate as u64;

    Ok(AudioBuffer { samples: all_samples, sample_rate, channels, duration_ms })
}
```

- [ ] **Step 6: Rodar testes**

```bash
cargo test audio_engine::decoder
```

Esperado: 2 testes PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/
git commit -m "feat: decodificador de áudio com symphonia (MP3/WAV/FLAC/AAC/OGG)"
```

---

### Task 3: Player de Áudio (Rust — `audio_engine/player.rs`)

**Files:**
- Create: `src-tauri/src/audio_engine/player.rs`
- Modify: `src-tauri/Cargo.toml` (adicionar cpal)

**Interfaces:**
- Consumes: `AudioBuffer` de `decoder.rs`
- Produces:
  - `AudioPlayer::new(buffer: AudioBuffer) -> AudioPlayer`
  - `AudioPlayer::play(&self)`
  - `AudioPlayer::pause(&self)`
  - `AudioPlayer::seek(&self, position_ms: u64)`
  - `AudioPlayer::position_ms(&self) -> u64`
  - `AudioPlayer::is_playing(&self) -> bool`

- [ ] **Step 1: Adicionar cpal ao `Cargo.toml`**

```toml
cpal = "0.15"
```

- [ ] **Step 2: Escrever testes em `player.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_engine::decoder::AudioBuffer;

    fn silent_buffer(duration_ms: u64) -> AudioBuffer {
        let sample_rate = 44100u32;
        let channels = 2u16;
        let frames = (sample_rate as u64 * duration_ms / 1000) as usize;
        AudioBuffer {
            samples: vec![0.0f32; frames * channels as usize],
            sample_rate,
            channels,
            duration_ms,
        }
    }

    #[test]
    fn test_player_starts_paused() {
        let player = AudioPlayer::new(silent_buffer(5000));
        assert!(!player.is_playing());
        assert_eq!(player.position_ms(), 0);
    }

    #[test]
    fn test_seek_clamps_to_duration() {
        let player = AudioPlayer::new(silent_buffer(5000));
        player.seek(99999);
        assert!(player.position_ms() <= 5000);
    }

    #[test]
    fn test_seek_updates_position() {
        let player = AudioPlayer::new(silent_buffer(10000));
        player.seek(3000);
        let pos = player.position_ms();
        assert!(pos >= 2900 && pos <= 3100);
    }
}
```

- [ ] **Step 3: Rodar para verificar falha**

```bash
cargo test audio_engine::player
```

Esperado: FAIL com `error[E0433]: failed to resolve`

- [ ] **Step 4: Implementar `AudioPlayer` em `player.rs`**

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use crate::audio_engine::decoder::AudioBuffer;

struct PlayerState {
    buffer: AudioBuffer,
    position_frames: usize,
    playing: bool,
}

pub struct AudioPlayer {
    state: Arc<Mutex<PlayerState>>,
    _stream: Option<cpal::Stream>,
}

impl AudioPlayer {
    pub fn new(buffer: AudioBuffer) -> Self {
        let state = Arc::new(Mutex::new(PlayerState {
            buffer,
            position_frames: 0,
            playing: false,
        }));

        let state_clone = Arc::clone(&state);
        let stream = Self::build_stream(state_clone).ok();

        AudioPlayer { state, _stream: stream }
    }

    fn build_stream(state: Arc<Mutex<PlayerState>>) -> Result<cpal::Stream, cpal::BuildStreamError> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or(cpal::BuildStreamError::DeviceNotAvailable)?;

        let (sample_rate, channels) = {
            let s = state.lock().unwrap();
            (s.buffer.sample_rate, s.buffer.channels as usize)
        };

        let config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                let mut s = state.lock().unwrap();
                if !s.playing {
                    output.fill(0.0);
                    return;
                }
                for sample in output.iter_mut() {
                    if s.position_frames < s.buffer.samples.len() {
                        *sample = s.buffer.samples[s.position_frames];
                        s.position_frames += 1;
                    } else {
                        *sample = 0.0;
                        s.playing = false;
                    }
                }
            },
            |_err| {},
            None,
        )?;

        stream.play().ok();
        Ok(stream)
    }

    pub fn play(&self) {
        let mut s = self.state.lock().unwrap();
        s.playing = true;
    }

    pub fn pause(&self) {
        let mut s = self.state.lock().unwrap();
        s.playing = false;
    }

    pub fn seek(&self, position_ms: u64) {
        let mut s = self.state.lock().unwrap();
        let total_frames = s.buffer.samples.len() / s.buffer.channels as usize;
        let target_frame = ((position_ms as f64 / 1000.0) * s.buffer.sample_rate as f64) as usize;
        s.position_frames = (target_frame * s.buffer.channels as usize).min(s.buffer.samples.len());
        let _ = total_frames;
    }

    pub fn position_ms(&self) -> u64 {
        let s = self.state.lock().unwrap();
        let frames = s.position_frames / s.buffer.channels.max(1) as usize;
        (frames as f64 / s.buffer.sample_rate as f64 * 1000.0) as u64
    }

    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().playing
    }
}
```

- [ ] **Step 5: Rodar testes**

```bash
cargo test audio_engine::player
```

Esperado: 3 testes PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/
git commit -m "feat: player de áudio com cpal (play/pause/seek/position)"
```

---

### Task 4: Gerenciador de Projetos (Rust — `project_manager/`)

**Files:**
- Create: `src-tauri/src/project_manager/mod.rs`
- Create: `src-tauri/src/project_manager/types.rs`
- Create: `src-tauri/src/project_manager/serializer.rs`
- Create: `src-tauri/src/project_manager/history.rs`
- Modify: `src-tauri/Cargo.toml` (adicionar zstd, serde_json)

**Interfaces:**
- Produces:
  - `Project { name, bpm, key, tracks: Vec<Track>, mixer: MixerSettings }`
  - `Track { id, name, stem_type, clip_path, volume, pan, muted, soloed }`
  - `save_project(project: &Project, path: &str) -> Result<(), ProjectError>`
  - `load_project(path: &str) -> Result<Project, ProjectError>`
  - `UndoRedoHistory<T>::push(state: T)`, `::undo() -> Option<T>`, `::redo() -> Option<T>`

- [ ] **Step 1: Adicionar dependências ao `Cargo.toml`**

```toml
serde_json = "1.0"
zstd = "0.13"
uuid = { version = "1.0", features = ["v4"] }
```

- [ ] **Step 2: Criar `types.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,
    pub bpm: f32,
    pub key: String,
    pub tracks: Vec<Track>,
    pub mixer: MixerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub stem_type: StemType,
    pub clip_path: Option<String>,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StemType {
    Vocals,
    Drums,
    Bass,
    Other,
    Guitar,
    Piano,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MixerSettings {
    pub master_volume: f32,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name: "Novo Projeto".to_string(),
            bpm: 120.0,
            key: "C".to_string(),
            tracks: vec![],
            mixer: MixerSettings { master_volume: 1.0 },
        }
    }
}
```

- [ ] **Step 3: Escrever testes em `serializer.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_manager::types::Project;

    #[test]
    fn test_save_and_load_roundtrip() {
        let project = Project::default();
        let path = std::env::temp_dir()
            .join("test_project.mrt")
            .to_str().unwrap().to_string();

        save_project(&project, &path).expect("deve salvar");
        let loaded = load_project(&path).expect("deve carregar");

        assert_eq!(project, loaded);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_missing_file_returns_error() {
        let result = load_project("/nao/existe.mrt");
        assert!(result.is_err());
    }

    #[test]
    fn test_saved_file_is_binary_compressed() {
        let project = Project::default();
        let path = std::env::temp_dir()
            .join("test_compressed.mrt")
            .to_str().unwrap().to_string();
        save_project(&project, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        // Arquivos zstd começam com magic number 0xFD2FB528
        assert_eq!(&bytes[..4], &[0x28, 0xB5, 0x2F, 0xFD]);
        std::fs::remove_file(&path).ok();
    }
}
```

- [ ] **Step 4: Rodar para verificar falha**

```bash
cargo test project_manager::serializer
```

Esperado: FAIL com `error[E0433]: failed to resolve`

- [ ] **Step 5: Implementar `serializer.rs`**

```rust
use crate::project_manager::types::Project;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("arquivo não encontrado: {0}")]
    FileNotFound(String),
    #[error("erro de serialização: {0}")]
    SerializationError(String),
    #[error("arquivo corrompido: {0}")]
    CorruptedFile(String),
}

pub fn save_project(project: &Project, path: &str) -> Result<(), ProjectError> {
    let json = serde_json::to_vec(project)
        .map_err(|e| ProjectError::SerializationError(e.to_string()))?;
    let compressed = zstd::encode_all(json.as_slice(), 3)
        .map_err(|e| ProjectError::SerializationError(e.to_string()))?;
    std::fs::write(path, compressed)
        .map_err(|e| ProjectError::SerializationError(e.to_string()))
}

pub fn load_project(path: &str) -> Result<Project, ProjectError> {
    let bytes = std::fs::read(path)
        .map_err(|_| ProjectError::FileNotFound(path.to_string()))?;
    let decompressed = zstd::decode_all(bytes.as_slice())
        .map_err(|e| ProjectError::CorruptedFile(e.to_string()))?;
    let project = serde_json::from_slice(&decompressed)
        .map_err(|e| ProjectError::CorruptedFile(e.to_string()))?;
    Ok(project)
}
```

- [ ] **Step 6: Implementar `history.rs`**

```rust
pub struct UndoRedoHistory<T: Clone> {
    states: Vec<T>,
    cursor: usize,
    max_size: usize,
}

impl<T: Clone> UndoRedoHistory<T> {
    pub fn new(initial: T) -> Self {
        UndoRedoHistory {
            states: vec![initial],
            cursor: 0,
            max_size: 100,
        }
    }

    pub fn push(&mut self, state: T) {
        self.states.truncate(self.cursor + 1);
        self.states.push(state);
        if self.states.len() > self.max_size + 1 {
            self.states.remove(0);
        } else {
            self.cursor = self.states.len() - 1;
        }
    }

    pub fn undo(&mut self) -> Option<T> {
        if self.cursor == 0 { return None; }
        self.cursor -= 1;
        Some(self.states[self.cursor].clone())
    }

    pub fn redo(&mut self) -> Option<T> {
        if self.cursor >= self.states.len() - 1 { return None; }
        self.cursor += 1;
        Some(self.states[self.cursor].clone())
    }

    pub fn current(&self) -> &T {
        &self.states[self.cursor]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo_basic() {
        let mut h = UndoRedoHistory::new(0i32);
        h.push(1);
        h.push(2);
        assert_eq!(*h.current(), 2);
        assert_eq!(h.undo(), Some(1));
        assert_eq!(h.undo(), Some(0));
        assert_eq!(h.undo(), None);
        assert_eq!(h.redo(), Some(1));
    }

    #[test]
    fn test_push_after_undo_clears_redo() {
        let mut h = UndoRedoHistory::new(0i32);
        h.push(1);
        h.push(2);
        h.undo();
        h.push(99);
        assert_eq!(h.redo(), None);
        assert_eq!(*h.current(), 99);
    }
}
```

- [ ] **Step 7: Criar `mod.rs`**

```rust
pub mod types;
pub mod serializer;
pub mod history;

pub use types::{Project, Track, StemType, MixerSettings};
pub use serializer::{save_project, load_project, ProjectError};
pub use history::UndoRedoHistory;
```

- [ ] **Step 8: Rodar todos os testes do project_manager**

```bash
cargo test project_manager
```

Esperado: 5 testes PASS.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/
git commit -m "feat: gerenciador de projetos .mrt com zstd + undo/redo"
```

---

### Task 5: Protocolo IPC e stem_bridge (Rust — `stem_bridge/`)

**Files:**
- Create: `src-tauri/src/stem_bridge/mod.rs`
- Create: `src-tauri/src/stem_bridge/protocol.rs`
- Create: `src-tauri/src/stem_bridge/ipc.rs`

**Interfaces:**
- Consumes: Python sidecar rodando em `stem_sidecar/main.py`
- Produces:
  - `StemBridge::new() -> StemBridge`
  - `StemBridge::send_request(req: StemRequest) -> Result<(), BridgeError>`
  - `StemBridge::read_event() -> Result<StemEvent, BridgeError>`
  - `StemRequest { audio_path, model, output_dir }`
  - `StemEvent::Progress { percent: u8, eta_seconds: u32 }` | `StemEvent::Done { stems: Vec<StemFile> }` | `StemEvent::Error { message }`

- [ ] **Step 1: Criar `protocol.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StemRequest {
    pub audio_path: String,
    pub model: String,        // "htdemucs" | "htdemucs_ft" | "htdemucs_6s"
    pub output_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StemEvent {
    Progress { percent: u8, eta_seconds: u32 },
    Done { stems: Vec<StemFile> },
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StemFile {
    pub stem_type: String,  // "vocals" | "drums" | "bass" | "other" | "guitar" | "piano"
    pub path: String,
}
```

- [ ] **Step 2: Escrever testes para o protocolo em `protocol.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stem_request_serializes() {
        let req = StemRequest {
            audio_path: "/tmp/song.mp3".to_string(),
            model: "htdemucs_ft".to_string(),
            output_dir: "/tmp/stems/".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("htdemucs_ft"));
        assert!(json.contains("audio_path"));
    }

    #[test]
    fn test_stem_event_progress_deserializes() {
        let json = r#"{"type":"Progress","percent":45,"eta_seconds":80}"#;
        let event: StemEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, StemEvent::Progress { percent: 45, .. }));
    }

    #[test]
    fn test_stem_event_done_deserializes() {
        let json = r#"{"type":"Done","stems":[{"stem_type":"vocals","path":"/tmp/vocals.wav"}]}"#;
        let event: StemEvent = serde_json::from_str(json).unwrap();
        if let StemEvent::Done { stems } = event {
            assert_eq!(stems[0].stem_type, "vocals");
        } else {
            panic!("esperado StemEvent::Done");
        }
    }
}
```

- [ ] **Step 3: Rodar para verificar falha**

```bash
cargo test stem_bridge::protocol
```

Esperado: FAIL com `error[E0433]`

- [ ] **Step 4: Implementar `ipc.rs`**

```rust
use std::io::{BufRead, BufReader, Write};
use thiserror::Error;
use crate::stem_bridge::protocol::{StemEvent, StemRequest};

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("IPC não disponível: {0}")]
    ConnectionError(String),
    #[error("timeout aguardando sidecar")]
    Timeout,
    #[error("erro de protocolo: {0}")]
    ProtocolError(String),
}

pub struct StemBridge {
    #[cfg(windows)]
    pipe_name: String,
    #[cfg(not(windows))]
    socket_path: String,
}

impl StemBridge {
    pub fn new() -> Self {
        #[cfg(windows)]
        return StemBridge { pipe_name: r"\\.\pipe\music-recort".to_string() };
        #[cfg(not(windows))]
        return StemBridge { socket_path: "/tmp/music-recort.sock".to_string() };
    }

    pub fn send_and_stream<F>(&self, req: StemRequest, mut on_event: F) -> Result<(), BridgeError>
    where
        F: FnMut(StemEvent),
    {
        #[cfg(windows)]
        {
            use std::fs::OpenOptions;
            let mut pipe = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&self.pipe_name)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let json = serde_json::to_string(&req)
                .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
            writeln!(pipe, "{}", json)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let reader = BufReader::new(pipe);
            for line in reader.lines() {
                let line = line.map_err(|e| BridgeError::ConnectionError(e.to_string()))?;
                if line.trim().is_empty() { continue; }
                let event: StemEvent = serde_json::from_str(&line)
                    .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
                let done = matches!(event, StemEvent::Done { .. } | StemEvent::Error { .. });
                on_event(event);
                if done { break; }
            }
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::net::UnixStream;
            let mut stream = UnixStream::connect(&self.socket_path)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let json = serde_json::to_string(&req)
                .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
            writeln!(stream, "{}", json)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let line = line.map_err(|e| BridgeError::ConnectionError(e.to_string()))?;
                if line.trim().is_empty() { continue; }
                let event: StemEvent = serde_json::from_str(&line)
                    .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
                let done = matches!(event, StemEvent::Done { .. } | StemEvent::Error { .. });
                on_event(event);
                if done { break; }
            }
        }
        Ok(())
    }
}
```

- [ ] **Step 5: Criar `mod.rs`**

```rust
pub mod protocol;
pub mod ipc;

pub use ipc::{StemBridge, BridgeError};
pub use protocol::{StemRequest, StemEvent, StemFile};
```

- [ ] **Step 6: Rodar testes de protocolo**

```bash
cargo test stem_bridge
```

Esperado: 3 testes PASS (apenas de protocolo — IPC requer sidecar ativo).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/
git commit -m "feat: stem_bridge IPC (named pipe / Unix socket + protocolo JSON)"
```

---

### Task 6: Tauri Commands (Rust — `commands/`)

**Files:**
- Create: `src-tauri/src/commands/audio.rs`
- Create: `src-tauri/src/commands/project.rs`
- Create: `src-tauri/src/commands/stems.rs`
- Create: `src-tauri/src/commands/mixer.rs`
- Modify: `src-tauri/src/lib.rs` (registrar commands)
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `AudioPlayer`, `save_project`, `load_project`, `StemBridge`, `StemRequest`
- Produces: Tauri Commands invocáveis pelo frontend

- [ ] **Step 1: Criar `commands/audio.rs`**

```rust
use std::sync::Mutex;
use tauri::State;
use crate::audio_engine::{decode_audio, AudioBuffer, AudioPlayer};
use serde::Serialize;

pub struct AudioState(pub Mutex<Option<AudioPlayer>>);

#[derive(Serialize)]
pub struct AudioInfo {
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
}

#[tauri::command]
pub fn load_file(path: String, state: State<AudioState>) -> Result<AudioInfo, String> {
    let buffer = decode_audio(&path).map_err(|e| e.to_string())?;
    let info = AudioInfo {
        duration_ms: buffer.duration_ms,
        sample_rate: buffer.sample_rate,
        channels: buffer.channels,
    };
    let player = AudioPlayer::new(buffer);
    *state.0.lock().unwrap() = Some(player);
    Ok(info)
}

#[tauri::command]
pub fn play(state: State<AudioState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    guard.as_ref().ok_or("nenhum arquivo carregado")?.play();
    Ok(())
}

#[tauri::command]
pub fn pause(state: State<AudioState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    guard.as_ref().ok_or("nenhum arquivo carregado")?.pause();
    Ok(())
}

#[tauri::command]
pub fn seek(position_ms: u64, state: State<AudioState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    guard.as_ref().ok_or("nenhum arquivo carregado")?.seek(position_ms);
    Ok(())
}

#[tauri::command]
pub fn get_position(state: State<AudioState>) -> u64 {
    state.0.lock().unwrap().as_ref().map(|p| p.position_ms()).unwrap_or(0)
}
```

- [ ] **Step 2: Criar `commands/project.rs`**

```rust
use std::sync::Mutex;
use tauri::State;
use crate::project_manager::{Project, save_project, load_project, UndoRedoHistory};
use serde::Serialize;

pub struct ProjectState(pub Mutex<UndoRedoHistory<Project>>);

#[tauri::command]
pub fn save(path: String, state: State<ProjectState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    save_project(guard.current(), &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load(path: String, state: State<ProjectState>) -> Result<Project, String> {
    let project = load_project(&path).map_err(|e| e.to_string())?;
    let mut guard = state.0.lock().unwrap();
    guard.push(project.clone());
    Ok(project)
}

#[tauri::command]
pub fn get_project(state: State<ProjectState>) -> Project {
    state.0.lock().unwrap().current().clone()
}

#[tauri::command]
pub fn undo(state: State<ProjectState>) -> Option<Project> {
    state.0.lock().unwrap().undo()
}

#[tauri::command]
pub fn redo(state: State<ProjectState>) -> Option<Project> {
    state.0.lock().unwrap().redo()
}
```

- [ ] **Step 3: Criar `commands/mixer.rs`**

```rust
use std::sync::Mutex;
use tauri::State;
use crate::project_manager::{Project, UndoRedoHistory};
use super::project::ProjectState;

#[tauri::command]
pub fn set_volume(track_id: String, volume: f32, state: State<ProjectState>) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let mut project = guard.current().clone();
    let track = project.tracks.iter_mut()
        .find(|t| t.id == track_id)
        .ok_or("faixa não encontrada")?;
    track.volume = volume.clamp(0.0, 2.0);
    guard.push(project);
    Ok(())
}

#[tauri::command]
pub fn set_pan(track_id: String, pan: f32, state: State<ProjectState>) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let mut project = guard.current().clone();
    let track = project.tracks.iter_mut()
        .find(|t| t.id == track_id)
        .ok_or("faixa não encontrada")?;
    track.pan = pan.clamp(-1.0, 1.0);
    guard.push(project);
    Ok(())
}
```

- [ ] **Step 4: Criar `commands/stems.rs`**

```rust
use tauri::{AppHandle, Emitter};
use crate::stem_bridge::{StemBridge, StemRequest, StemEvent};

#[tauri::command]
pub async fn separate(
    audio_path: String,
    model: String,
    output_dir: String,
    app: AppHandle,
) -> Result<(), String> {
    let bridge = StemBridge::new();
    let req = StemRequest { audio_path, model, output_dir };

    std::thread::spawn(move || {
        let result = bridge.send_and_stream(req, |event| {
            match &event {
                StemEvent::Progress { percent, eta_seconds } => {
                    app.emit("stem:progress", serde_json::json!({
                        "percent": percent,
                        "eta_seconds": eta_seconds
                    })).ok();
                }
                StemEvent::Done { stems } => {
                    app.emit("stem:done", stems).ok();
                }
                StemEvent::Error { message } => {
                    app.emit("stem:error", message).ok();
                }
            }
        });
        if let Err(e) = result {
            app.emit("stem:error", e.to_string()).ok();
        }
    });

    Ok(())
}
```

- [ ] **Step 5: Atualizar `lib.rs`**

```rust
mod audio_engine;
mod project_manager;
mod stem_bridge;
mod commands;

use commands::{audio::*, project::*, mixer::*, stems::*};
use commands::audio::AudioState;
use commands::project::ProjectState;
use project_manager::{Project, UndoRedoHistory};
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AudioState(Mutex::new(None)))
        .manage(ProjectState(Mutex::new(UndoRedoHistory::new(Project::default()))))
        .invoke_handler(tauri::generate_handler![
            load_file, play, pause, seek, get_position,
            save, load, get_project, undo, redo,
            set_volume, set_pan,
            separate,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao rodar Tauri");
}
```

- [ ] **Step 6: Compilar e verificar que não há erros**

```bash
cargo tauri build --debug 2>&1 | tail -20
```

Esperado: `Finished debug [unoptimized + debuginfo] target(s)`

- [ ] **Step 7: Commit**

```bash
git add src-tauri/
git commit -m "feat: Tauri commands (audio, projeto, mixer, stems)"
```

---

### Task 7: Frontend — lib/tauri.ts e Stores Zustand

**Files:**
- Create: `src/lib/tauri.ts`
- Create: `src/stores/audioStore.ts`
- Create: `src/stores/projectStore.ts`
- Create: `src/stores/stemStore.ts`

**Interfaces:**
- Produces:
  - `invoke.loadFile(path)`, `invoke.play()`, `invoke.pause()`, `invoke.seek(ms)`
  - `useAudioStore()` → `{ durationMs, positionMs, isPlaying, audioInfo }`
  - `useProjectStore()` → `{ project, setProject, updateTrackVolume, updateTrackPan }`
  - `useStemStore()` → `{ stems, progress, separating, error }`

- [ ] **Step 1: Criar `src/lib/tauri.ts`**

```typescript
import { invoke as tauriInvoke } from '@tauri-apps/api/core'

export interface AudioInfo {
  duration_ms: number
  sample_rate: number
  channels: number
}

export interface Track {
  id: string
  name: string
  stem_type: string
  clip_path: string | null
  volume: number
  pan: number
  muted: boolean
  soloed: boolean
}

export interface Project {
  name: string
  bpm: number
  key: string
  tracks: Track[]
  mixer: { master_volume: number }
}

export interface StemFile {
  stem_type: string
  path: string
}

export const invoke = {
  loadFile: (path: string) => tauriInvoke<AudioInfo>('load_file', { path }),
  play: () => tauriInvoke<void>('play'),
  pause: () => tauriInvoke<void>('pause'),
  seek: (positionMs: number) => tauriInvoke<void>('seek', { position_ms: positionMs }),
  getPosition: () => tauriInvoke<number>('get_position'),
  saveProject: (path: string) => tauriInvoke<void>('save', { path }),
  loadProject: (path: string) => tauriInvoke<Project>('load', { path }),
  getProject: () => tauriInvoke<Project>('get_project'),
  undo: () => tauriInvoke<Project | null>('undo'),
  redo: () => tauriInvoke<Project | null>('redo'),
  setVolume: (trackId: string, volume: number) =>
    tauriInvoke<void>('set_volume', { track_id: trackId, volume }),
  setPan: (trackId: string, pan: number) =>
    tauriInvoke<void>('set_pan', { track_id: trackId, pan }),
  separate: (audioPath: string, model: string, outputDir: string) =>
    tauriInvoke<void>('separate', { audio_path: audioPath, model, output_dir: outputDir }),
}
```

- [ ] **Step 2: Criar `src/stores/audioStore.ts`**

```typescript
import { create } from 'zustand'
import { AudioInfo } from '../lib/tauri'

interface AudioStore {
  audioInfo: AudioInfo | null
  isPlaying: boolean
  positionMs: number
  setAudioInfo: (info: AudioInfo) => void
  setIsPlaying: (playing: boolean) => void
  setPositionMs: (ms: number) => void
}

export const useAudioStore = create<AudioStore>((set) => ({
  audioInfo: null,
  isPlaying: false,
  positionMs: 0,
  setAudioInfo: (audioInfo) => set({ audioInfo }),
  setIsPlaying: (isPlaying) => set({ isPlaying }),
  setPositionMs: (positionMs) => set({ positionMs }),
}))
```

- [ ] **Step 3: Criar `src/stores/projectStore.ts`**

```typescript
import { create } from 'zustand'
import { Project } from '../lib/tauri'

interface ProjectStore {
  project: Project
  isDirty: boolean
  setProject: (project: Project) => void
  updateTrackVolume: (trackId: string, volume: number) => void
  updateTrackPan: (trackId: string, pan: number) => void
  markClean: () => void
}

const defaultProject: Project = {
  name: 'Novo Projeto',
  bpm: 120,
  key: 'C',
  tracks: [],
  mixer: { master_volume: 1.0 },
}

export const useProjectStore = create<ProjectStore>((set) => ({
  project: defaultProject,
  isDirty: false,
  setProject: (project) => set({ project, isDirty: true }),
  updateTrackVolume: (trackId, volume) =>
    set((state) => ({
      isDirty: true,
      project: {
        ...state.project,
        tracks: state.project.tracks.map((t) =>
          t.id === trackId ? { ...t, volume } : t
        ),
      },
    })),
  updateTrackPan: (trackId, pan) =>
    set((state) => ({
      isDirty: true,
      project: {
        ...state.project,
        tracks: state.project.tracks.map((t) =>
          t.id === trackId ? { ...t, pan } : t
        ),
      },
    })),
  markClean: () => set({ isDirty: false }),
}))
```

- [ ] **Step 4: Criar `src/stores/stemStore.ts`**

```typescript
import { create } from 'zustand'
import { StemFile } from '../lib/tauri'

interface StemStore {
  stems: StemFile[]
  progress: number
  etaSeconds: number
  separating: boolean
  error: string | null
  startSeparation: () => void
  setProgress: (percent: number, eta: number) => void
  setStemsDone: (stems: StemFile[]) => void
  setError: (message: string) => void
  reset: () => void
}

export const useStemStore = create<StemStore>((set) => ({
  stems: [],
  progress: 0,
  etaSeconds: 0,
  separating: false,
  error: null,
  startSeparation: () => set({ separating: true, progress: 0, error: null, stems: [] }),
  setProgress: (progress, etaSeconds) => set({ progress, etaSeconds }),
  setStemsDone: (stems) => set({ stems, separating: false, progress: 100 }),
  setError: (error) => set({ error, separating: false }),
  reset: () => set({ stems: [], progress: 0, separating: false, error: null }),
}))
```

- [ ] **Step 5: Verificar que TypeScript compila sem erros**

```bash
npx tsc --noEmit
```

Esperado: sem erros.

- [ ] **Step 6: Commit**

```bash
git add src/
git commit -m "feat: lib/tauri.ts tipado + stores Zustand (audio, projeto, stems)"
```

---

### Task 8: Hooks React

**Files:**
- Create: `src/hooks/useAudioEngine.ts`
- Create: `src/hooks/useProject.ts`
- Create: `src/hooks/useStemSeparation.ts`

**Interfaces:**
- Consumes: `invoke` de `lib/tauri.ts`, todos os stores
- Produces:
  - `useAudioEngine()` → `{ loadFile, play, pause, seek, positionMs, isPlaying, audioInfo }`
  - `useProject()` → `{ project, loadProject, saveProject }`
  - `useStemSeparation()` → `{ separate, stems, progress, separating, error }`

- [ ] **Step 1: Criar `src/hooks/useAudioEngine.ts`**

```typescript
import { useCallback, useEffect, useRef } from 'react'
import { invoke } from '../lib/tauri'
import { useAudioStore } from '../stores/audioStore'

export function useAudioEngine() {
  const { audioInfo, isPlaying, positionMs, setAudioInfo, setIsPlaying, setPositionMs } =
    useAudioStore()
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const startPolling = useCallback(() => {
    intervalRef.current = setInterval(async () => {
      const pos = await invoke.getPosition()
      setPositionMs(pos)
    }, 100)
  }, [setPositionMs])

  const stopPolling = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
  }, [])

  const loadFile = useCallback(async (path: string) => {
    const info = await invoke.loadFile(path)
    setAudioInfo(info)
    setPositionMs(0)
    setIsPlaying(false)
  }, [setAudioInfo, setPositionMs, setIsPlaying])

  const play = useCallback(async () => {
    await invoke.play()
    setIsPlaying(true)
    startPolling()
  }, [setIsPlaying, startPolling])

  const pause = useCallback(async () => {
    await invoke.pause()
    setIsPlaying(false)
    stopPolling()
  }, [setIsPlaying, stopPolling])

  const seek = useCallback(async (ms: number) => {
    await invoke.seek(ms)
    setPositionMs(ms)
  }, [setPositionMs])

  useEffect(() => () => stopPolling(), [stopPolling])

  return { loadFile, play, pause, seek, positionMs, isPlaying, audioInfo }
}
```

- [ ] **Step 2: Criar `src/hooks/useProject.ts`**

```typescript
import { useCallback, useEffect } from 'react'
import { invoke } from '../lib/tauri'
import { useProjectStore } from '../stores/projectStore'

export function useProject() {
  const { project, isDirty, setProject, markClean } = useProjectStore()

  // Auto-save a cada 30 segundos se houver mudanças
  useEffect(() => {
    if (!isDirty) return
    const timer = setTimeout(async () => {
      try {
        await invoke.saveProject(`${project.name}.mrt`)
        markClean()
      } catch {
        // silencioso — próximo ciclo tenta de novo
      }
    }, 30_000)
    return () => clearTimeout(timer)
  }, [isDirty, project.name, markClean])

  const loadProject = useCallback(async (path: string) => {
    const loaded = await invoke.loadProject(path)
    setProject(loaded)
    markClean()
  }, [setProject, markClean])

  const saveProject = useCallback(async (path: string) => {
    await invoke.saveProject(path)
    markClean()
  }, [markClean])

  return { project, isDirty, loadProject, saveProject }
}
```

- [ ] **Step 3: Criar `src/hooks/useStemSeparation.ts`**

```typescript
import { useCallback, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke, StemFile } from '../lib/tauri'
import { useStemStore } from '../stores/stemStore'
import { useProjectStore } from '../stores/projectStore'
import { useAudioStore } from '../stores/audioStore'

export function useStemSeparation() {
  const { stems, progress, etaSeconds, separating, error,
          startSeparation, setProgress, setStemsDone, setError } = useStemStore()
  const { setProject, project } = useProjectStore()

  useEffect(() => {
    const unlistenProgress = listen<{ percent: number; eta_seconds: number }>(
      'stem:progress',
      (e) => setProgress(e.payload.percent, e.payload.eta_seconds)
    )
    const unlistenDone = listen<StemFile[]>('stem:done', (e) => {
      setStemsDone(e.payload)
      // Adiciona stems como faixas no projeto
      const newTracks = e.payload.map((s) => ({
        id: `${s.stem_type}-${Date.now()}`,
        name: s.stem_type.charAt(0).toUpperCase() + s.stem_type.slice(1),
        stem_type: s.stem_type,
        clip_path: s.path,
        volume: 1.0,
        pan: 0.0,
        muted: false,
        soloed: false,
      }))
      setProject({ ...project, tracks: [...project.tracks, ...newTracks] })
    })
    const unlistenError = listen<string>('stem:error', (e) => setError(e.payload))

    return () => {
      unlistenProgress.then((f) => f())
      unlistenDone.then((f) => f())
      unlistenError.then((f) => f())
    }
  }, [setProgress, setStemsDone, setError, setProject, project])

  const separate = useCallback(async (audioPath: string, model = 'htdemucs_ft') => {
    startSeparation()
    const outputDir = audioPath.replace(/[^/\\]+$/, 'stems')
    await invoke.separate(audioPath, model, outputDir)
  }, [startSeparation])

  return { separate, stems, progress, etaSeconds, separating, error }
}
```

- [ ] **Step 4: Verificar TypeScript**

```bash
npx tsc --noEmit
```

Esperado: sem erros.

- [ ] **Step 5: Commit**

```bash
git add src/
git commit -m "feat: hooks useAudioEngine, useProject, useStemSeparation"
```

---

### Task 9: Componentes React — Transport e Waveform

**Files:**
- Create: `src/components/Transport/Transport.tsx`
- Create: `src/components/Waveform/Waveform.tsx`

**Interfaces:**
- Consumes: `useAudioEngine`
- Produces: componentes visuais de controle e visualização de áudio

- [ ] **Step 1: Criar `src/components/Transport/Transport.tsx`**

```tsx
import { useAudioEngine } from '../../hooks/useAudioEngine'

function formatTime(ms: number): string {
  const totalSec = Math.floor(ms / 1000)
  const min = Math.floor(totalSec / 60)
  const sec = totalSec % 60
  return `${String(min).padStart(2, '0')}:${String(sec).padStart(2, '0')}`
}

export function Transport() {
  const { play, pause, seek, positionMs, isPlaying, audioInfo } = useAudioEngine()
  const duration = audioInfo?.duration_ms ?? 0

  return (
    <div className="flex items-center gap-4 p-3 bg-[#16213e] border-b border-purple-900">
      <button
        onClick={isPlaying ? pause : play}
        disabled={!audioInfo}
        className="w-10 h-10 rounded-full bg-purple-600 hover:bg-purple-500 disabled:opacity-30
                   flex items-center justify-center text-white font-bold transition-colors"
      >
        {isPlaying ? '⏸' : '▶'}
      </button>

      <button
        onClick={() => seek(0)}
        disabled={!audioInfo}
        className="w-8 h-8 rounded bg-gray-700 hover:bg-gray-600 disabled:opacity-30
                   flex items-center justify-center text-white text-sm transition-colors"
      >
        ⏹
      </button>

      <span className="text-purple-300 font-mono text-sm w-24">
        {formatTime(positionMs)} / {formatTime(duration)}
      </span>

      <input
        type="range"
        min={0}
        max={duration}
        value={positionMs}
        onChange={(e) => seek(Number(e.target.value))}
        disabled={!audioInfo}
        className="flex-1 accent-purple-500 disabled:opacity-30"
      />
    </div>
  )
}
```

- [ ] **Step 2: Criar `src/components/Waveform/Waveform.tsx`**

```tsx
import { useEffect, useRef } from 'react'
import { useAudioStore } from '../../stores/audioStore'

interface WaveformProps {
  samples?: Float32Array
  width?: number
  height?: number
}

export function Waveform({ samples, width = 800, height = 80 }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const positionMs = useAudioStore((s) => s.positionMs)
  const audioInfo = useAudioStore((s) => s.audioInfo)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !samples) return
    const ctx = canvas.getContext('2d')!
    ctx.clearRect(0, 0, width, height)

    // Fundo
    ctx.fillStyle = '#0f0f1a'
    ctx.fillRect(0, 0, width, height)

    // Waveform
    const step = Math.ceil(samples.length / width)
    const mid = height / 2

    ctx.beginPath()
    ctx.strokeStyle = '#7c3aed'
    ctx.lineWidth = 1

    for (let x = 0; x < width; x++) {
      const start = x * step
      let min = 1, max = -1
      for (let i = start; i < start + step && i < samples.length; i++) {
        if (samples[i] < min) min = samples[i]
        if (samples[i] > max) max = samples[i]
      }
      ctx.moveTo(x, mid + min * mid)
      ctx.lineTo(x, mid + max * mid)
    }
    ctx.stroke()

    // Playhead
    if (audioInfo && audioInfo.duration_ms > 0) {
      const playheadX = (positionMs / audioInfo.duration_ms) * width
      ctx.beginPath()
      ctx.strokeStyle = '#f0abfc'
      ctx.lineWidth = 2
      ctx.moveTo(playheadX, 0)
      ctx.lineTo(playheadX, height)
      ctx.stroke()
    }
  }, [samples, positionMs, audioInfo, width, height])

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      className="w-full rounded bg-[#0f0f1a]"
    />
  )
}
```

- [ ] **Step 3: Verificar TypeScript**

```bash
npx tsc --noEmit
```

Esperado: sem erros.

- [ ] **Step 4: Commit**

```bash
git add src/
git commit -m "feat: componentes Transport (play/pause/seek) e Waveform (canvas 2D)"
```

---

### Task 10: Componentes React — StemPanel e Mixer

**Files:**
- Create: `src/components/StemPanel/StemPanel.tsx`
- Create: `src/components/Mixer/Mixer.tsx`
- Create: `src/components/Timeline/Timeline.tsx`
- Modify: `src/App.tsx` (montar layout completo)

**Interfaces:**
- Consumes: `useStemSeparation`, `useProjectStore`, `invoke.setVolume`, `invoke.setPan`

- [ ] **Step 1: Criar `src/components/StemPanel/StemPanel.tsx`**

```tsx
import { useStemSeparation } from '../../hooks/useStemSeparation'

const STEM_COLORS: Record<string, string> = {
  vocals: 'bg-pink-600',
  drums: 'bg-orange-600',
  bass: 'bg-blue-600',
  other: 'bg-green-600',
  guitar: 'bg-yellow-600',
  piano: 'bg-purple-600',
}

export function StemPanel({ audioPath }: { audioPath: string | null }) {
  const { separate, stems, progress, etaSeconds, separating, error } = useStemSeparation()

  return (
    <div className="p-4 bg-[#16213e] rounded border border-purple-900">
      <h2 className="text-purple-300 font-semibold mb-3">Separação de Stems</h2>

      {!separating && stems.length === 0 && (
        <div className="flex gap-2">
          {(['htdemucs', 'htdemucs_ft', 'htdemucs_6s'] as const).map((model) => (
            <button
              key={model}
              onClick={() => audioPath && separate(audioPath, model)}
              disabled={!audioPath}
              className="px-3 py-1.5 text-xs rounded bg-purple-700 hover:bg-purple-600
                         disabled:opacity-30 text-white transition-colors"
            >
              {model}
            </button>
          ))}
        </div>
      )}

      {separating && (
        <div className="mt-2">
          <div className="flex justify-between text-xs text-gray-400 mb-1">
            <span>Processando... {progress}%</span>
            <span>ETA: {etaSeconds}s</span>
          </div>
          <div className="w-full bg-gray-700 rounded-full h-2">
            <div
              className="bg-purple-500 h-2 rounded-full transition-all"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
      )}

      {error && (
        <p className="text-red-400 text-xs mt-2">{error}</p>
      )}

      {stems.length > 0 && (
        <div className="mt-3 flex flex-wrap gap-2">
          {stems.map((s) => (
            <span
              key={s.stem_type}
              className={`px-2 py-1 text-xs rounded text-white ${STEM_COLORS[s.stem_type] ?? 'bg-gray-600'}`}
            >
              {s.stem_type}
            </span>
          ))}
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Criar `src/components/Mixer/Mixer.tsx`**

```tsx
import { useProjectStore } from '../../stores/projectStore'
import { invoke } from '../../lib/tauri'

export function Mixer() {
  const { project, updateTrackVolume, updateTrackPan } = useProjectStore()

  if (project.tracks.length === 0) {
    return (
      <div className="p-4 bg-[#16213e] rounded border border-purple-900 text-gray-500 text-sm">
        Nenhuma faixa. Importe um áudio e separe os stems.
      </div>
    )
  }

  return (
    <div className="p-4 bg-[#16213e] rounded border border-purple-900">
      <h2 className="text-purple-300 font-semibold mb-3">Mixer</h2>
      <div className="flex gap-4 overflow-x-auto">
        {project.tracks.map((track) => (
          <div key={track.id} className="flex flex-col items-center gap-2 min-w-[60px]">
            <span className="text-xs text-gray-300 truncate w-14 text-center">
              {track.name}
            </span>

            {/* Fader de volume (vertical) */}
            <input
              type="range"
              min={0}
              max={2}
              step={0.01}
              value={track.volume}
              onChange={async (e) => {
                const vol = Number(e.target.value)
                updateTrackVolume(track.id, vol)
                await invoke.setVolume(track.id, vol)
              }}
              className="h-24 accent-purple-500"
              style={{ writingMode: 'vertical-lr', direction: 'rtl' }}
            />

            <span className="text-xs text-purple-400 font-mono">
              {Math.round(track.volume * 100)}
            </span>

            {/* Pan */}
            <input
              type="range"
              min={-1}
              max={1}
              step={0.01}
              value={track.pan}
              onChange={async (e) => {
                const pan = Number(e.target.value)
                updateTrackPan(track.id, pan)
                await invoke.setPan(track.id, pan)
              }}
              className="w-14 accent-pink-400"
            />
          </div>
        ))}
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Criar `src/components/Timeline/Timeline.tsx`**

```tsx
import { useProjectStore } from '../../stores/projectStore'
import { Waveform } from '../Waveform/Waveform'

export function Timeline() {
  const { project } = useProjectStore()

  return (
    <div className="flex-1 overflow-y-auto bg-[#0d0d1a] p-2 space-y-1">
      {project.tracks.map((track) => (
        <div key={track.id} className="flex items-center gap-2">
          <div className="w-24 shrink-0 text-xs text-gray-300 truncate bg-[#16213e] px-2 py-1 rounded">
            {track.name}
          </div>
          <div className="flex-1">
            <Waveform height={48} />
          </div>
        </div>
      ))}
      {project.tracks.length === 0 && (
        <div className="flex items-center justify-center h-32 text-gray-600 text-sm">
          Importe um arquivo de áudio para começar
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 4: Montar layout em `src/App.tsx`**

```tsx
import { useState, useCallback } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { Transport } from './components/Transport/Transport'
import { Timeline } from './components/Timeline/Timeline'
import { StemPanel } from './components/StemPanel/StemPanel'
import { Mixer } from './components/Mixer/Mixer'
import { useAudioEngine } from './hooks/useAudioEngine'

export default function App() {
  const [audioPath, setAudioPath] = useState<string | null>(null)
  const { loadFile } = useAudioEngine()

  const handleImport = useCallback(async () => {
    const selected = await open({
      filters: [{ name: 'Áudio', extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg'] }],
    })
    if (typeof selected === 'string') {
      setAudioPath(selected)
      await loadFile(selected)
    }
  }, [loadFile])

  return (
    <div className="flex flex-col h-screen bg-[#1a1a2e] text-gray-100">
      {/* Barra de menu */}
      <div className="flex items-center gap-3 px-4 py-2 bg-[#0d0d1a] border-b border-purple-900">
        <span className="text-purple-400 font-bold text-sm">Music Recort</span>
        <button
          onClick={handleImport}
          className="px-3 py-1 text-xs rounded bg-purple-700 hover:bg-purple-600 text-white transition-colors"
        >
          Importar Áudio
        </button>
        {audioPath && (
          <span className="text-gray-500 text-xs truncate max-w-xs">
            {audioPath.split(/[/\\]/).pop()}
          </span>
        )}
      </div>

      {/* Transport */}
      <Transport />

      {/* Timeline */}
      <Timeline />

      {/* Painel inferior: StemPanel + Mixer */}
      <div className="flex gap-2 p-2 bg-[#0d0d1a] border-t border-purple-900 max-h-56 overflow-y-auto">
        <div className="flex-1">
          <StemPanel audioPath={audioPath} />
        </div>
        <div className="flex-1">
          <Mixer />
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 5: Instalar plugin de dialog do Tauri**

```bash
npm install @tauri-apps/plugin-dialog
cargo add tauri-plugin-dialog
```

Adicionar ao `lib.rs` antes de `.invoke_handler`:
```rust
.plugin(tauri_plugin_dialog::init())
```

- [ ] **Step 6: Rodar o app e testar importação**

```bash
cargo tauri dev
```

Esperado: janela com barra de menu, botão "Importar Áudio", transport e timeline vazios.

- [ ] **Step 7: Commit**

```bash
git add src/ src-tauri/
git commit -m "feat: layout completo — StemPanel, Mixer, Timeline, App shell"
```

---

### Task 11: Python Sidecar — IPC Loop e Demucs

**Files:**
- Create: `stem_sidecar/requirements.txt`
- Create: `stem_sidecar/main.py`
- Create: `stem_sidecar/separator.py`
- Create: `stem_sidecar/api_fallback.py`
- Create: `stem_sidecar/preprocessor.py`

**Interfaces:**
- Consumes: mensagem JSON `StemRequest` via stdin / named pipe
- Produces: mensagens JSON `StemEvent` (Progress, Done, Error) via stdout / named pipe

- [ ] **Step 1: Criar `stem_sidecar/requirements.txt`**

```
demucs==4.0.1
librosa==0.10.2
soundfile==0.12.1
torch>=2.0.0
torchaudio>=2.0.0
requests==2.31.0
```

- [ ] **Step 2: Criar `stem_sidecar/preprocessor.py`**

```python
import librosa
import json

def analyze_audio(path: str) -> dict:
    """Detecta BPM e tonalidade do arquivo de áudio."""
    try:
        y, sr = librosa.load(path, sr=None, mono=True, duration=60.0)
        tempo, _ = librosa.beat.beat_track(y=y, sr=sr)
        chroma = librosa.feature.chroma_cqt(y=y, sr=sr)
        key_idx = int(chroma.mean(axis=1).argmax())
        keys = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
        return {"bpm": float(round(tempo, 1)), "key": keys[key_idx]}
    except Exception as e:
        return {"bpm": 120.0, "key": "C", "error": str(e)}
```

- [ ] **Step 3: Criar `stem_sidecar/separator.py`**

```python
import subprocess
import sys
import os
import json
import torch
from pathlib import Path

MODELS = ["htdemucs", "htdemucs_ft", "htdemucs_6s"]
STEM_NAMES = {
    "htdemucs": ["vocals", "drums", "bass", "other"],
    "htdemucs_ft": ["vocals", "drums", "bass", "other"],
    "htdemucs_6s": ["vocals", "drums", "bass", "other", "guitar", "piano"],
}

def has_gpu() -> bool:
    return torch.cuda.is_available()

def separate_stems(audio_path: str, model: str, output_dir: str, progress_cb) -> list[dict]:
    """
    Separa stems usando Demucs.
    progress_cb(percent: int, eta_seconds: int) chamado periodicamente.
    Retorna lista de {"stem_type": str, "path": str}.
    """
    if model not in MODELS:
        model = "htdemucs_ft"

    Path(output_dir).mkdir(parents=True, exist_ok=True)

    device = "cuda" if has_gpu() else "cpu"

    # Demucs via subprocess para capturar progresso
    cmd = [
        sys.executable, "-m", "demucs",
        "--name", model,
        "--device", device,
        "--out", output_dir,
        audio_path,
    ]

    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )

    percent = 0
    for line in process.stdout:
        line = line.strip()
        # Demucs imprime "X%" no stderr/stdout durante processamento
        if "%" in line:
            try:
                pct_str = [t for t in line.split() if "%" in t][0]
                percent = int(pct_str.replace("%", ""))
                # ETA estimada simples baseada no progresso
                eta = max(0, int((100 - percent) * 1.5))
                progress_cb(percent, eta)
            except (ValueError, IndexError):
                pass

    process.wait()

    if process.returncode != 0:
        raise RuntimeError(f"Demucs falhou com código {process.returncode}")

    # Localizar stems gerados
    audio_name = Path(audio_path).stem
    stems_dir = Path(output_dir) / model / audio_name
    stems = []
    for stem_type in STEM_NAMES.get(model, []):
        stem_path = stems_dir / f"{stem_type}.wav"
        if stem_path.exists():
            stems.append({"stem_type": stem_type, "path": str(stem_path)})

    return stems
```

- [ ] **Step 4: Criar `stem_sidecar/api_fallback.py`**

```python
import requests
import os
import time

MOISES_API_KEY = os.environ.get("MOISES_API_KEY", "")
MOISES_API_URL = "https://developer-api.moises.ai/api/job"

def separate_via_api(audio_path: str, output_dir: str, progress_cb) -> list[dict]:
    """
    Fallback: usa a API do Moises para separação quando GPU não disponível.
    Requer MOISES_API_KEY definido como variável de ambiente.
    """
    if not MOISES_API_KEY:
        raise RuntimeError(
            "MOISES_API_KEY não definida. Configure a variável de ambiente ou use o modelo local."
        )

    headers = {"Authorization": MOISES_API_KEY, "Content-Type": "application/json"}

    # 1. Criar job
    payload = {
        "name": "stem-separation",
        "workflow": "moises/stem-4",
        "params": {"inputUrl": f"file://{audio_path}"},
    }
    resp = requests.post(MOISES_API_URL, json=payload, headers=headers, timeout=30)
    resp.raise_for_status()
    job_id = resp.json()["id"]

    # 2. Polling até concluir
    for i in range(360):  # máximo 30 minutos
        time.sleep(5)
        status_resp = requests.get(f"{MOISES_API_URL}/{job_id}", headers=headers, timeout=30)
        status_resp.raise_for_status()
        data = status_resp.json()
        status = data.get("status")
        progress_cb(min(i, 95), max(0, 300 - i * 5))
        if status == "SUCCEEDED":
            # Baixar stems
            import pathlib, urllib.request
            pathlib.Path(output_dir).mkdir(parents=True, exist_ok=True)
            stems = []
            for key, url in data.get("result", {}).items():
                dest = str(pathlib.Path(output_dir) / f"{key}.wav")
                urllib.request.urlretrieve(url, dest)
                stems.append({"stem_type": key, "path": dest})
            return stems
        elif status in ("FAILED", "CANCELLED"):
            raise RuntimeError(f"Job API falhou: {status}")

    raise RuntimeError("Timeout aguardando API de separação")
```

- [ ] **Step 5: Criar `stem_sidecar/main.py`**

```python
"""
Loop IPC do sidecar de separação de stems.
Lê StemRequest JSON de stdin (uma linha), emite StemEvent JSON para stdout.
"""
import sys
import json
import os
import torch
from separator import separate_stems
from api_fallback import separate_via_api
from preprocessor import analyze_audio

def emit(event: dict):
    print(json.dumps(event), flush=True)

def handle_request(req: dict):
    audio_path = req["audio_path"]
    model = req.get("model", "htdemucs_ft")
    output_dir = req["output_dir"]

    def progress_cb(percent: int, eta_seconds: int):
        emit({"type": "Progress", "percent": percent, "eta_seconds": eta_seconds})

    # Detectar se deve usar API ou local
    use_api = not torch.cuda.is_available()
    if use_api:
        try:
            import librosa
            duration = librosa.get_duration(path=audio_path)
            use_api = duration > 600  # > 10 minutos
        except Exception:
            use_api = False

    try:
        if use_api:
            stems = separate_via_api(audio_path, output_dir, progress_cb)
        else:
            stems = separate_stems(audio_path, model, output_dir, progress_cb)

        emit({"type": "Done", "stems": stems})
    except Exception as e:
        emit({"type": "Error", "message": str(e)})

def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
            handle_request(req)
        except json.JSONDecodeError as e:
            emit({"type": "Error", "message": f"JSON inválido: {e}"})

if __name__ == "__main__":
    main()
```

- [ ] **Step 6: Testar sidecar manualmente**

```bash
cd stem_sidecar
pip install -r requirements.txt

# Teste de smoke (sem arquivo real — deve retornar erro controlado)
echo '{"audio_path":"/nao/existe.mp3","model":"htdemucs_ft","output_dir":"/tmp/stems"}' | python main.py
```

Esperado: `{"type": "Error", "message": "..."}` — erro controlado de arquivo não encontrado.

- [ ] **Step 7: Testar com arquivo real (se disponível)**

```bash
# Coloque um arquivo MP3 de teste em ~/test.mp3
echo '{"audio_path":"C:/Users/jaderson.moreira/test.mp3","model":"htdemucs","output_dir":"C:/Users/jaderson.moreira/stems_test"}' | python main.py
```

Esperado: stream de `Progress` seguido de `Done` com lista de stems.

- [ ] **Step 8: Commit**

```bash
cd ..
git add stem_sidecar/
git commit -m "feat: Python sidecar — Demucs local + fallback API + IPC JSON"
```

---

### Task 12: Integração Final e Teste End-to-End

**Files:**
- Modify: `src-tauri/tauri.conf.json` (configurar sidecar)
- Modify: `src-tauri/Cargo.toml` (adicionar tauri-plugin-shell para sidecar)

**Interfaces:**
- Consumes: todos os componentes anteriores
- Produces: aplicação funcional end-to-end

- [ ] **Step 1: Configurar sidecar no `tauri.conf.json`**

```json
{
  "bundle": {
    "externalBin": ["../stem_sidecar/main"]
  },
  "app": {
    "security": {
      "capabilities": ["default"]
    }
  }
}
```

- [ ] **Step 2: Adicionar plugin shell ao Rust**

```bash
cargo add tauri-plugin-shell
```

Em `lib.rs`, adicionar antes de `.invoke_handler`:
```rust
.plugin(tauri_plugin_shell::init())
```

- [ ] **Step 3: Criar script de setup do ambiente Python**

```bash
# stem_sidecar/setup.sh
#!/bin/bash
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
echo "Sidecar pronto. Ative com: source stem_sidecar/.venv/bin/activate"
```

Para Windows, criar `stem_sidecar/setup.ps1`:
```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
Write-Host "Sidecar pronto."
```

- [ ] **Step 4: Testar fluxo completo**

```bash
# Terminal 1 — iniciar sidecar manualmente para teste
cd stem_sidecar
.\.venv\Scripts\Activate.ps1
python main.py

# Terminal 2 — rodar app
cargo tauri dev
```

Fluxo de teste:
1. Clicar "Importar Áudio" → selecionar MP3
2. Verificar que Transport mostra duração correta
3. Clicar Play → verificar que áudio toca
4. No StemPanel, clicar "htdemucs" → verificar barra de progresso
5. Após conclusão, verificar que stems aparecem como faixas na Timeline e no Mixer
6. Mover fader de volume de uma faixa → verificar que atualiza

- [ ] **Step 5: Verificar auto-save**

Aguardar 30 segundos com projeto modificado. Verificar que arquivo `.mrt` é criado no diretório do projeto.

- [ ] **Step 6: Commit final**

```bash
git add .
git commit -m "feat: integração completa Fases 1 e 2 — DAW + separação de stems"
```

---

## Checklist de Revisão do Spec

| Requisito do Spec | Task que implementa |
|-------------------|---------------------|
| Importação MP3/WAV/FLAC/AAC/OGG | Task 2 (decoder.rs) |
| Playback + controles de transporte | Task 3 (player.rs) + Task 9 (Transport.tsx) |
| Renderização de waveform | Task 9 (Waveform.tsx) |
| Gerenciamento de projetos .mrt | Task 4 (project_manager/) |
| Auto-save 30 segundos | Task 8 (useProject.ts) |
| Undo/redo até 100 ações | Task 4 (history.rs) |
| Demucs local (3 modelos) | Task 11 (separator.py) |
| Fallback API (sem GPU + >10min) | Task 11 (main.py) + Task 11 (api_fallback.py) |
| Progress streaming | Task 5 (stem_bridge) + Task 11 (main.py) + Task 8 (useStemSeparation.ts) |
| Stems como faixas na timeline | Task 8 (useStemSeparation.ts) |
| Mixer (volume, pan) | Task 10 (Mixer.tsx) + Task 6 (commands/mixer.rs) |
| Detecção BPM/tonalidade | Task 11 (preprocessor.py) |
| Named pipes Windows / Unix sockets Mac | Task 5 (ipc.rs) |
| Formato .mrt = JSON + zstd | Task 4 (serializer.rs) |
