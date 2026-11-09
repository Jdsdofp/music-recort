use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StemRequest {
    pub audio_path: String,
    pub model: String,        // "htdemucs" | "htdemucs_ft" | "htdemucs_6s"
    pub output_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StemEvent {
    Progress { percent: u8, eta_seconds: u32 },
    Done { stems: Vec<StemFile> },
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StemFile {
    pub stem_type: String,  // "vocals" | "drums" | "bass" | "other" | "guitar" | "piano"
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stem_request_serializes() {
        let req = StemRequest {
            audio_path: "/tmp/song.mp3".to_string(),
            model: "htdemucs_ft".to_string(),
            output_dir: "/tmp/stems/".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("htdemucs_ft"));
        assert!(json.contains("audio_path"));
    }

    #[test]
    fn test_stem_event_progress_deserializes() {
        let json = r#"{"type":"Progress","percent":45,"eta_seconds":80}"#;
        let event: StemEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, StemEvent::Progress { percent: 45, .. }));
    }

    #[test]
    fn test_stem_event_done_deserializes() {
        let json = r#"{"type":"Done","stems":[{"stem_type":"vocals","path":"/tmp/vocals.wav"}]}"#;
        let event: StemEvent = serde_json::from_str(json).unwrap();
        if let StemEvent::Done { stems } = event {
            assert_eq!(stems[0].stem_type, "vocals");
        } else {
            panic!("esperado StemEvent::Done");
        }
    }
}
