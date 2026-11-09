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
