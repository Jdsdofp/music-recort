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
