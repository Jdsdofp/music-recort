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

  const loadFile = useCallback(
    async (path: string) => {
      const info = await invoke.loadFile(path)
      setAudioInfo(info)
      setPositionMs(0)
      setIsPlaying(false)
    },
    [setAudioInfo, setPositionMs, setIsPlaying],
  )

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

  const seek = useCallback(
    async (ms: number) => {
      await invoke.seek(ms)
      setPositionMs(ms)
    },
    [setPositionMs],
  )

  useEffect(() => () => stopPolling(), [stopPolling])

  return { loadFile, play, pause, seek, positionMs, isPlaying, audioInfo }
}
