use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use crate::stem_bridge::protocol::{StemEvent, StemRequest};

#[derive(Debug)]
pub enum BridgeError {
    ConnectionError(String),
    Timeout,
    ProtocolError(String),
    ProcessError(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::ConnectionError(msg) => write!(f, "IPC não disponível: {}", msg),
            BridgeError::Timeout => write!(f, "timeout aguardando sidecar"),
            BridgeError::ProtocolError(msg) => write!(f, "erro de protocolo: {}", msg),
            BridgeError::ProcessError(msg) => write!(f, "erro no processo sidecar: {}", msg),
        }
    }
}

impl std::error::Error for BridgeError {}

/// Localiza o executável Python tentando `python`, `python3`, `py` em ordem.
fn find_python() -> Option<String> {
    for candidate in &["python", "python3", "py"] {
        let result = Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if result.map(|s| s.success()).unwrap_or(false) {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Localiza o script main.py do sidecar.
/// Tenta, em ordem:
///   1. Relativo ao executável: `../stem_sidecar/main.py`
///   2. Relativo ao diretório atual: `stem_sidecar/main.py`
fn find_sidecar_path() -> Option<std::path::PathBuf> {
    // Tentativa 1: relativo ao executável
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join("..").join("stem_sidecar").join("main.py");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // Tentativa 2: relativo ao diretório atual (desenvolvimento)
    let candidate = std::path::Path::new("stem_sidecar").join("main.py");
    if candidate.exists() {
        return Some(candidate);
    }

    None
}

pub struct StemBridge;

impl StemBridge {
    pub fn new() -> Self {
        StemBridge
    }

    pub fn send_and_stream<F>(&self, req: StemRequest, mut on_event: F) -> Result<(), BridgeError>
    where
        F: FnMut(StemEvent),
    {
        let python = find_python()
            .ok_or_else(|| BridgeError::ConnectionError("Python não encontrado (tentou: python, python3, py)".to_string()))?;

        let sidecar_path = find_sidecar_path()
            .ok_or_else(|| BridgeError::ConnectionError("stem_sidecar/main.py não encontrado".to_string()))?;

        let mut child = Command::new(&python)
            .arg(&sidecar_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| BridgeError::ProcessError(format!("falha ao iniciar sidecar: {}", e)))?;

        // Escrever a requisição no stdin
        {
            let stdin = child.stdin.as_mut()
                .ok_or_else(|| BridgeError::ProcessError("stdin do sidecar não disponível".to_string()))?;

            let json = serde_json::to_string(&req)
                .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
            writeln!(stdin, "{}", json)
                .map_err(|e| BridgeError::ProcessError(format!("erro ao escrever no stdin: {}", e)))?;
        }
        // stdin é fechado aqui (dropped), sinalizando EOF para o Python

        // Ler eventos do stdout com timeout de 5 minutos
        let stdout = child.stdout.take()
            .ok_or_else(|| BridgeError::ProcessError("stdout do sidecar não disponível".to_string()))?;

        let reader = BufReader::new(stdout);
        let start = Instant::now();
        let timeout = Duration::from_secs(300);

        for line in reader.lines() {
            if start.elapsed() > timeout {
                let _ = child.kill();
                return Err(BridgeError::Timeout);
            }

            let line = line.map_err(|e| BridgeError::ProcessError(format!("erro ao ler stdout: {}", e)))?;
            if line.trim().is_empty() {
                continue;
            }

            let event: StemEvent = serde_json::from_str(&line)
                .map_err(|e| BridgeError::ProtocolError(format!("{} (linha: {})", e, line)))?;
            let done = matches!(event, StemEvent::Done { .. } | StemEvent::Error { .. });
            on_event(event);
            if done {
                break;
            }
        }

        // Aguardar o processo terminar
        let _ = child.wait();

        Ok(())
    }
}
