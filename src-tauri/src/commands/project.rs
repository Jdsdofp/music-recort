use std::sync::Mutex;
use tauri::State;
use crate::project_manager::{Project, save_project, load_project, UndoRedoHistory};

pub struct ProjectState(pub Mutex<UndoRedoHistory<Project>>);

#[tauri::command]
pub fn save(path: String, state: State<ProjectState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    save_project(guard.current(), &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load(path: String, state: State<ProjectState>) -> Result<Project, String> {
    let project = load_project(&path).map_err(|e| e.to_string())?;
    let mut guard = state.0.lock().unwrap();
    guard.push(project.clone());
    Ok(project)
}

#[tauri::command]
pub fn get_project(state: State<ProjectState>) -> Project {
    state.0.lock().unwrap().current().clone()
}

#[tauri::command]
pub fn undo(state: State<ProjectState>) -> Option<Project> {
    state.0.lock().unwrap().undo()
}

#[tauri::command]
pub fn redo(state: State<ProjectState>) -> Option<Project> {
    state.0.lock().unwrap().redo()
}
