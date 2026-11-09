import { useProjectStore } from '../../stores/projectStore'
import { Waveform } from '../Waveform/Waveform'

export function Timeline() {
  const { project } = useProjectStore()

  return (
    <div className="flex-1 overflow-y-auto bg-[#0d0d1a] p-2 space-y-1">
      {project.tracks.length === 0 && (
        <div className="flex items-center justify-center h-32 text-gray-600 text-sm">
          Importe um arquivo de áudio para começar
        </div>
      )}
      {project.tracks.map((track) => (
        <div key={track.id} className="flex items-center gap-2">
          <div
            className="shrink-0 text-xs text-gray-300 truncate bg-[#16213e] px-2 py-1 rounded border border-purple-900"
            style={{ width: 96, height: 48, display: 'flex', alignItems: 'center' }}
            title={track.name}
          >
            {track.name}
          </div>
          <div className="flex-1">
            <Waveform height={48} />
          </div>
        </div>
      ))}
    </div>
  )
}
