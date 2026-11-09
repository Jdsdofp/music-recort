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
