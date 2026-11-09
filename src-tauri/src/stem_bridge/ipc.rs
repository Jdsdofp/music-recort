use std::io::{BufRead, BufReader, Write};
use thiserror::Error;
use crate::stem_bridge::protocol::{StemEvent, StemRequest};

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("IPC não disponível: {0}")]
    ConnectionError(String),
    #[error("timeout aguardando sidecar")]
    Timeout,
    #[error("erro de protocolo: {0}")]
    ProtocolError(String),
}

pub struct StemBridge {
    #[cfg(windows)]
    pipe_name: String,
    #[cfg(not(windows))]
    socket_path: String,
}

impl StemBridge {
    pub fn new() -> Self {
        #[cfg(windows)]
        return StemBridge {
            pipe_name: r"\\.\pipe\music-recort".to_string(),
        };
        #[cfg(not(windows))]
        return StemBridge {
            socket_path: "/tmp/music-recort.sock".to_string(),
        };
    }

    pub fn send_and_stream<F>(&self, req: StemRequest, mut on_event: F) -> Result<(), BridgeError>
    where
        F: FnMut(StemEvent),
    {
        #[cfg(windows)]
        {
            use std::fs::OpenOptions;
            let mut pipe = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&self.pipe_name)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let json = serde_json::to_string(&req)
                .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
            writeln!(pipe, "{}", json)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let reader = BufReader::new(pipe);
            for line in reader.lines() {
                let line = line.map_err(|e| BridgeError::ConnectionError(e.to_string()))?;
                if line.trim().is_empty() {
                    continue;
                }
                let event: StemEvent = serde_json::from_str(&line)
                    .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
                let done = matches!(event, StemEvent::Done { .. } | StemEvent::Error { .. });
                on_event(event);
                if done {
                    break;
                }
            }
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::net::UnixStream;
            let mut stream = UnixStream::connect(&self.socket_path)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let json = serde_json::to_string(&req)
                .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
            writeln!(stream, "{}", json)
                .map_err(|e| BridgeError::ConnectionError(e.to_string()))?;

            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let line = line.map_err(|e| BridgeError::ConnectionError(e.to_string()))?;
                if line.trim().is_empty() {
                    continue;
                }
                let event: StemEvent = serde_json::from_str(&line)
                    .map_err(|e| BridgeError::ProtocolError(e.to_string()))?;
                let done = matches!(event, StemEvent::Done { .. } | StemEvent::Error { .. });
                on_event(event);
                if done {
                    break;
                }
            }
        }
        Ok(())
    }
}
