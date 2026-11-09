import { useStemSeparation } from '../../hooks/useStemSeparation'

const STEM_COLORS: Record<string, string> = {
  vocals: 'bg-pink-600',
  drums: 'bg-orange-600',
  bass: 'bg-blue-600',
  other: 'bg-green-600',
  guitar: 'bg-yellow-600',
  piano: 'bg-purple-600',
}

export function StemPanel({ audioPath }: { audioPath: string | null }) {
  const { separate, stems, progress, etaSeconds, separating, error } = useStemSeparation()

  return (
    <div className="p-4 bg-[#16213e] rounded border border-purple-900 h-full">
      <h2 className="text-purple-300 font-semibold mb-3 text-sm">Separação de Stems</h2>

      {!separating && stems.length === 0 && (
        <div className="flex flex-wrap gap-2">
          {(['htdemucs', 'htdemucs_ft', 'htdemucs_6s'] as const).map((model) => (
            <button
              key={model}
              onClick={() => audioPath && separate(audioPath, model)}
              disabled={!audioPath}
              className="px-3 py-1.5 text-xs rounded bg-purple-700 hover:bg-purple-600
                         disabled:opacity-30 text-white transition-colors"
            >
              {model}
            </button>
          ))}
        </div>
      )}

      {!audioPath && stems.length === 0 && !separating && (
        <p className="text-gray-600 text-xs mt-2">Importe um áudio para habilitar a separação.</p>
      )}

      {separating && (
        <div className="mt-2">
          <div className="flex justify-between text-xs text-gray-400 mb-1">
            <span>Processando... {progress}%</span>
            <span>ETA: {etaSeconds}s</span>
          </div>
          <div className="w-full bg-gray-700 rounded-full h-2">
            <div
              className="bg-purple-500 h-2 rounded-full transition-all"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
      )}

      {error && (
        <p className="text-red-400 text-xs mt-2">{error}</p>
      )}

      {stems.length > 0 && (
        <div className="mt-3 flex flex-wrap gap-2">
          {stems.map((s) => (
            <span
              key={s.stem_type}
              className={`px-2 py-1 text-xs rounded text-white ${STEM_COLORS[s.stem_type] ?? 'bg-gray-600'}`}
            >
              {s.stem_type}
            </span>
          ))}
        </div>
      )}
    </div>
  )
}
