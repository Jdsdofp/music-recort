pub mod audio_engine;
pub mod project_manager;
pub mod stem_bridge;
pub mod commands;

use commands::audio::{AudioState, load_file, play, pause, seek, get_position};
use commands::project::{ProjectState, save, load, get_project, undo, redo};
use commands::mixer::{set_volume, set_pan};
use commands::stems::separate;
use project_manager::{Project, UndoRedoHistory};
use std::sync::Mutex;

#[cfg(not(test))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AudioState(Mutex::new(None)))
        .manage(ProjectState(Mutex::new(UndoRedoHistory::new(Project::default()))))
        .invoke_handler(tauri::generate_handler![
            load_file, play, pause, seek, get_position,
            save, load, get_project, undo, redo,
            set_volume, set_pan,
            separate,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao rodar Tauri");
}
