use std::sync::Mutex;
use tauri::State;
use crate::audio_engine::{decode_audio, AudioPlayer};
use serde::Serialize;

pub struct AudioState(pub Mutex<Option<AudioPlayer>>);

// SAFETY: AudioPlayer contém cpal::Stream que não implementa Send no Windows,
// mas o acesso é sempre serializado pelo Mutex — nunca há acesso concorrente.
unsafe impl Send for AudioState {}
unsafe impl Sync for AudioState {}

#[derive(Serialize)]
pub struct AudioInfo {
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
}

#[tauri::command]
pub fn load_file(path: String, state: State<AudioState>) -> Result<AudioInfo, String> {
    let buffer = decode_audio(&path).map_err(|e| e.to_string())?;
    let info = AudioInfo {
        duration_ms: buffer.duration_ms,
        sample_rate: buffer.sample_rate,
        channels: buffer.channels,
    };
    let player = AudioPlayer::new(buffer);
    *state.0.lock().unwrap() = Some(player);
    Ok(info)
}

#[tauri::command]
pub fn play(state: State<AudioState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    guard.as_ref().ok_or("nenhum arquivo carregado")?.play();
    Ok(())
}

#[tauri::command]
pub fn pause(state: State<AudioState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    guard.as_ref().ok_or("nenhum arquivo carregado")?.pause();
    Ok(())
}

#[tauri::command]
pub fn seek(position_ms: u64, state: State<AudioState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    guard.as_ref().ok_or("nenhum arquivo carregado")?.seek(position_ms);
    Ok(())
}

#[tauri::command]
pub fn get_position(state: State<AudioState>) -> u64 {
    state.0.lock().unwrap().as_ref().map(|p| p.position_ms()).unwrap_or(0)
}
