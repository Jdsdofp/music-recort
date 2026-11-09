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
