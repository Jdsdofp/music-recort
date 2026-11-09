import { useCallback, useEffect } from 'react'
import { invoke } from '../lib/tauri'
import { useProjectStore } from '../stores/projectStore'

export function useProject() {
  const { project, isDirty, setProject, markClean } = useProjectStore()

  // Auto-save a cada 30 segundos se houver mudanças pendentes
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

  const loadProject = useCallback(
    async (path: string) => {
      const loaded = await invoke.loadProject(path)
      setProject(loaded)
      markClean()
    },
    [setProject, markClean],
  )

  const saveProject = useCallback(
    async (path: string) => {
      await invoke.saveProject(path)
      markClean()
    },
    [markClean],
  )

  return { project, isDirty, loadProject, saveProject }
}
