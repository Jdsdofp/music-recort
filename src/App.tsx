import { useState, useCallback } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { Transport } from './components/Transport/Transport'
import { Timeline } from './components/Timeline/Timeline'
import { StemPanel } from './components/StemPanel/StemPanel'
import { Mixer } from './components/Mixer/Mixer'
import { useAudioEngine } from './hooks/useAudioEngine'
import { useProject } from './hooks/useProject'
import { useStemSeparation } from './hooks/useStemSeparation'

export default function App() {
  const [audioPath, setAudioPath] = useState<string | null>(null)
  const { loadFile } = useAudioEngine()

  // Registra auto-save de 30 s e listeners de stem:progress/done/error
  useProject()
  useStemSeparation()

  const handleImport = useCallback(async () => {
    const selected = await open({
      filters: [{ name: 'Áudio', extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg'] }],
    })
    if (typeof selected === 'string') {
      setAudioPath(selected)
      await loadFile(selected)
    }
  }, [loadFile])

  return (
    <div className="flex flex-col h-screen bg-[#1a1a2e] text-gray-100 overflow-hidden">
      {/* Barra de menu */}
      <div className="flex items-center gap-3 px-4 py-2 bg-[#0d0d1a] border-b border-purple-900 shrink-0">
        <span className="text-purple-400 font-bold text-sm">Music Recort</span>
        <button
          onClick={handleImport}
          className="px-3 py-1 text-xs rounded bg-purple-700 hover:bg-purple-600 text-white transition-colors"
        >
          Importar Áudio
        </button>
        {audioPath && (
          <span className="text-gray-500 text-xs truncate max-w-xs">
            {audioPath.split(/[/\\]/).pop()}
          </span>
        )}
      </div>

      {/* Transport */}
      <div className="shrink-0">
        <Transport />
      </div>

      {/* Timeline — expande para preencher o espaço disponível */}
      <Timeline />

      {/* Painel inferior: StemPanel + Mixer */}
      <div className="flex gap-2 p-2 bg-[#0d0d1a] border-t border-purple-900 shrink-0 h-48">
        <div className="flex-1 overflow-hidden">
          <StemPanel audioPath={audioPath} />
        </div>
        <div className="flex-1 overflow-hidden">
          <Mixer />
        </div>
      </div>
    </div>
  )
}
