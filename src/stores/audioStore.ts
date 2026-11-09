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
