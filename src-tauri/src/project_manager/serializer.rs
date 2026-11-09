use crate::project_manager::types::Project;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("arquivo não encontrado: {0}")]
    FileNotFound(String),
    #[error("erro de serialização: {0}")]
    SerializationError(String),
    #[error("arquivo corrompido: {0}")]
    CorruptedFile(String),
}

pub fn save_project(project: &Project, path: &str) -> Result<(), ProjectError> {
    let json = serde_json::to_vec(project)
        .map_err(|e| ProjectError::SerializationError(e.to_string()))?;
    let compressed = zstd::encode_all(json.as_slice(), 3)
        .map_err(|e| ProjectError::SerializationError(e.to_string()))?;
    std::fs::write(path, compressed)
        .map_err(|e| ProjectError::SerializationError(e.to_string()))
}

pub fn load_project(path: &str) -> Result<Project, ProjectError> {
    let bytes = std::fs::read(path)
        .map_err(|_| ProjectError::FileNotFound(path.to_string()))?;
    let decompressed = zstd::decode_all(bytes.as_slice())
        .map_err(|e| ProjectError::CorruptedFile(e.to_string()))?;
    let project = serde_json::from_slice(&decompressed)
        .map_err(|e| ProjectError::CorruptedFile(e.to_string()))?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_manager::types::Project;

    #[test]
    fn test_save_and_load_roundtrip() {
        let project = Project::default();
        let path = std::env::temp_dir()
            .join("test_project.mrt")
            .to_str()
            .unwrap()
            .to_string();

        save_project(&project, &path).expect("deve salvar");
        let loaded = load_project(&path).expect("deve carregar");

        assert_eq!(project, loaded);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_missing_file_returns_error() {
        let result = load_project("/nao/existe.mrt");
        assert!(result.is_err());
        matches!(result.unwrap_err(), ProjectError::FileNotFound(_));
    }

    #[test]
    fn test_saved_file_is_binary_compressed() {
        let project = Project::default();
        let path = std::env::temp_dir()
            .join("test_compressed.mrt")
            .to_str()
            .unwrap()
            .to_string();
        save_project(&project, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        // Arquivos zstd começam com magic number 0xFD2FB528 (little-endian: 0x28, 0xB5, 0x2F, 0xFD)
        assert_eq!(&bytes[..4], &[0x28, 0xB5, 0x2F, 0xFD]);
        std::fs::remove_file(&path).ok();
    }
}
