use tauri::{AppHandle, Emitter};
use crate::stem_bridge::{StemBridge, StemRequest, StemEvent};

#[tauri::command]
pub async fn separate(
    audio_path: String,
    model: String,
    output_dir: String,
    app: AppHandle,
) -> Result<(), String> {
    let bridge = StemBridge::new();
    let req = StemRequest { audio_path, model, output_dir };

    std::thread::spawn(move || {
        let result = bridge.send_and_stream(req, |event| {
            match &event {
                StemEvent::Progress { percent, eta_seconds } => {
                    app.emit("stem:progress", serde_json::json!({
                        "percent": percent,
                        "eta_seconds": eta_seconds
                    })).ok();
                }
                StemEvent::Done { stems } => {
                    app.emit("stem:done", stems).ok();
                }
                StemEvent::Error { message } => {
                    app.emit("stem:error", message).ok();
                }
            }
        });
        if let Err(e) = result {
            app.emit("stem:error", e.to_string()).ok();
        }
    });

    Ok(())
}
