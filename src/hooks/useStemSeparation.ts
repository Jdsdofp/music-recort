import { useCallback, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke, StemFile } from '../lib/tauri'
import { useStemStore } from '../stores/stemStore'
import { useProjectStore } from '../stores/projectStore'

export function useStemSeparation() {
  const { stems, progress, etaSeconds, separating, error, startSeparation, setProgress, setStemsDone, setError } =
    useStemStore()
  // Usar o updater funcional do store diretamente para evitar stale closure em project
  const setProjectFn = useProjectStore((state) => state.setProject)
  const getProject = useProjectStore.getState

  useEffect(() => {
    const unlistenProgress = listen<{ percent: number; eta_seconds: number }>(
      'stem:progress',
      (e) => setProgress(e.payload.percent, e.payload.eta_seconds),
    )

    const unlistenDone = listen<StemFile[]>('stem:done', (e) => {
      setStemsDone(e.payload)
      // Adiciona stems como novas faixas no projeto usando getState para evitar stale closure
      const currentProject = getProject().project
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
      setProjectFn({ ...currentProject, tracks: [...currentProject.tracks, ...newTracks] })
    })

    const unlistenError = listen<string>('stem:error', (e) => setError(e.payload))

    return () => {
      unlistenProgress.then((f) => f())
      unlistenDone.then((f) => f())
      unlistenError.then((f) => f())
    }
  }, [setProgress, setStemsDone, setError, setProjectFn, getProject])

  const separate = useCallback(
    async (audioPath: string, model = 'htdemucs_ft') => {
      startSeparation()
      const outputDir = audioPath.replace(/[^/\\]+$/, 'stems')
      await invoke.separate(audioPath, model, outputDir)
    },
    [startSeparation],
  )

  return { separate, stems, progress, etaSeconds, separating, error }
}
