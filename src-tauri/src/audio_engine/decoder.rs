use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("arquivo não encontrado: {0}")]
    FileNotFound(String),
    #[error("formato não suportado: {0}")]
    UnsupportedFormat(String),
    #[error("erro de decodificação: {0}")]
    DecodingError(String),
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
}

pub fn decode_audio(path: &str) -> Result<AudioBuffer, AudioError> {
    let file = std::fs::File::open(path)
        .map_err(|_| AudioError::FileNotFound(path.to_string()))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| AudioError::UnsupportedFormat("nenhuma faixa de áudio".to_string()))?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| AudioError::UnsupportedFormat(e.to_string()))?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(_)) | Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(AudioError::DecodingError(e.to_string())),
        };
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let spec = *audio_buf.spec();
                let mut sample_buf = SampleBuffer::<f32>::new(audio_buf.capacity() as u64, spec);
                sample_buf.copy_interleaved_ref(audio_buf);
                all_samples.extend_from_slice(sample_buf.samples());
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(AudioError::DecodingError(e.to_string())),
        }
    }

    let total_frames = all_samples.len() as u64 / channels as u64;
    let duration_ms = (total_frames * 1000) / sample_rate as u64;

    Ok(AudioBuffer { samples: all_samples, sample_rate, channels, duration_ms })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_decode_returns_error_for_missing_file() {
        let result = decode_audio("/nao/existe.mp3");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AudioError::FileNotFound(_)));
    }

    #[test]
    fn test_audio_buffer_has_valid_fields() {
        // Cria WAV de teste sintético (1 segundo, 44100Hz, mono, silêncio)
        let path = create_test_wav();
        let buf = decode_audio(&path).expect("deve decodificar WAV de teste");
        assert_eq!(buf.sample_rate, 44100);
        assert_eq!(buf.channels, 1);
        assert!(buf.duration_ms >= 900 && buf.duration_ms <= 1100);
        assert!(!buf.samples.is_empty());
        std::fs::remove_file(&path).ok();
    }

    fn create_test_wav() -> String {
        let path = std::env::temp_dir().join("test_audio.wav");
        let mut f = std::fs::File::create(&path).unwrap();
        let num_samples: u32 = 44100;
        let data_size = num_samples * 4;
        let file_size = 36 + data_size;
        f.write_all(b"RIFF").unwrap();
        f.write_all(&file_size.to_le_bytes()).unwrap();
        f.write_all(b"WAVEfmt ").unwrap();
        f.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
        f.write_all(&3u16.to_le_bytes()).unwrap();  // PCM float
        f.write_all(&1u16.to_le_bytes()).unwrap();  // mono
        f.write_all(&44100u32.to_le_bytes()).unwrap();
        f.write_all(&(44100u32 * 4).to_le_bytes()).unwrap();
        f.write_all(&4u16.to_le_bytes()).unwrap();  // block align
        f.write_all(&32u16.to_le_bytes()).unwrap(); // bits per sample
        f.write_all(b"data").unwrap();
        f.write_all(&data_size.to_le_bytes()).unwrap();
        for _ in 0..num_samples {
            f.write_all(&0.0f32.to_le_bytes()).unwrap();
        }
        path.to_str().unwrap().to_string()
    }
}
