import { useEffect, useRef } from 'react'
import { useAudioStore } from '../../stores/audioStore'

interface WaveformProps {
  samples?: Float32Array
  width?: number
  height?: number
}

export function Waveform({ samples, width = 800, height = 80 }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const positionMs = useAudioStore((s) => s.positionMs)
  const audioInfo = useAudioStore((s) => s.audioInfo)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    ctx.clearRect(0, 0, width, height)

    // Fundo
    ctx.fillStyle = '#0f0f1a'
    ctx.fillRect(0, 0, width, height)

    // Waveform
    if (samples && samples.length > 0) {
      const step = Math.ceil(samples.length / width)
      const mid = height / 2

      ctx.beginPath()
      ctx.strokeStyle = '#7c3aed'
      ctx.lineWidth = 1

      for (let x = 0; x < width; x++) {
        const start = x * step
        let min = 1
        let max = -1
        for (let i = start; i < start + step && i < samples.length; i++) {
          const sample = samples[i]
          if (sample < min) min = sample
          if (sample > max) max = sample
        }
        ctx.moveTo(x, mid + min * mid)
        ctx.lineTo(x, mid + max * mid)
      }
      ctx.stroke()
    }

    // Playhead
    if (audioInfo && audioInfo.duration_ms > 0) {
      const playheadX = (positionMs / audioInfo.duration_ms) * width
      ctx.beginPath()
      ctx.strokeStyle = '#f0abfc'
      ctx.lineWidth = 2
      ctx.moveTo(playheadX, 0)
      ctx.lineTo(playheadX, height)
      ctx.stroke()
    }
  }, [samples, positionMs, audioInfo, width, height])

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      className="w-full rounded bg-[#0f0f1a]"
    />
  )
}
