import { useProjectStore } from '../../stores/projectStore'
import { invoke } from '../../lib/tauri'

export function Mixer() {
  const { project, updateTrackVolume, updateTrackPan } = useProjectStore()

  if (project.tracks.length === 0) {
    return (
      <div className="p-4 bg-[#16213e] rounded border border-purple-900 h-full text-gray-500 text-sm flex items-center">
        Nenhuma faixa. Importe um áudio e separe os stems.
      </div>
    )
  }

  return (
    <div className="p-4 bg-[#16213e] rounded border border-purple-900 h-full">
      <h2 className="text-purple-300 font-semibold mb-3 text-sm">Mixer</h2>
      <div className="flex gap-4 overflow-x-auto">
        {project.tracks.map((track) => (
          <div key={track.id} className="flex flex-col items-center gap-2 min-w-[60px]">
            <span className="text-xs text-gray-300 truncate w-14 text-center" title={track.name}>
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
              className="h-24 accent-purple-500 cursor-pointer"
              style={{ writingMode: 'vertical-lr', direction: 'rtl' }}
            />

            <span className="text-xs text-purple-400 font-mono">
              {Math.round(track.volume * 100)}%
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
              className="w-14 accent-pink-400 cursor-pointer"
            />

            <span className="text-xs text-gray-500 font-mono">
              {track.pan === 0 ? 'C' : track.pan > 0 ? `R${Math.round(track.pan * 100)}` : `L${Math.round(Math.abs(track.pan) * 100)}`}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}
