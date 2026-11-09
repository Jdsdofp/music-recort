use tauri::State;
use super::project::ProjectState;

#[tauri::command]
pub fn set_volume(track_id: String, volume: f32, state: State<ProjectState>) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let mut project = guard.current().clone();
    let track = project.tracks.iter_mut()
        .find(|t| t.id == track_id)
        .ok_or("faixa não encontrada")?;
    track.volume = volume.clamp(0.0, 2.0);
    guard.push(project);
    Ok(())
}

#[tauri::command]
pub fn set_pan(track_id: String, pan: f32, state: State<ProjectState>) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let mut project = guard.current().clone();
    let track = project.tracks.iter_mut()
        .find(|t| t.id == track_id)
        .ok_or("faixa não encontrada")?;
    track.pan = pan.clamp(-1.0, 1.0);
    guard.push(project);
    Ok(())
}
