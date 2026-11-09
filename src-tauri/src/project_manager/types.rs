use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,
    pub bpm: f32,
    pub key: String,
    pub tracks: Vec<Track>,
    pub mixer: MixerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub stem_type: StemType,
    pub clip_path: Option<String>,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StemType {
    Vocals,
    Drums,
    Bass,
    Other,
    Guitar,
    Piano,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MixerSettings {
    pub master_volume: f32,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            name: "Novo Projeto".to_string(),
            bpm: 120.0,
            key: "C".to_string(),
            tracks: vec![],
            mixer: MixerSettings { master_volume: 1.0 },
        }
    }
}
