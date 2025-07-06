use std::fmt::Display;

#[derive(Debug)]
pub struct PklError {
    pub message: String,
    pub trace: Option<String>,
}

impl PklError {
    pub fn parse(raw: String) -> Self {
        let parts = raw.splitn(3, '\n').collect::<Vec<_>>();

        if parts.len() < 2 {
            return Self {
                message: raw,
                trace: None,
            };
        }

        Self {
            message: parts[1].to_string(),
            trace: Some(parts[2].trim().to_string()),
        }
    }
}

impl Display for PklError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PklError occurred")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("failed to parse JSON: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ValueError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("unexpected value detected")]
    UnexpectedValue,
    #[error("failed to read value: {0}")]
    Read(#[from] rmp::decode::ValueReadError),
    #[error("failed to read string: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("failed to read marker: {0:?}")]
    MarkerRead(rmp::decode::MarkerReadError<std::io::Error>),
    #[error("invalid marker: {0:?}")]
    InvalidMarker(rmp::Marker),
}

impl From<rmp::decode::MarkerReadError<std::io::Error>> for ValueError {
    fn from(e: rmp::decode::MarkerReadError<std::io::Error>) -> Self {
        ValueError::MarkerRead(e)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("PklError: {0}")]
    Pkl(PklError),
    #[error("failed to decode value: {0}")]
    Value(#[from] ValueError),
    #[error("invalid request ID: expected {expected}, got {actual}")]
    InvalidRequestId { expected: u64, actual: u64 },
    #[error("failed to encode: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("failed to decode: {0}")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("invalid code: {0}")]
    InvalidCode(u64),
    #[error("invalid marker: {0:?}")]
    InvalidMarker(rmp::Marker),
    #[error("invalid response: {0}")]
    InvalidResponse(&'static str),
    #[error("failed to read marker: {0:?}")]
    MarkerRead(rmp::decode::MarkerReadError<std::io::Error>),
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("stdin/stdout not present")]
    Pipe,
}

impl From<rmp::decode::MarkerReadError<std::io::Error>> for Error {
    fn from(e: rmp::decode::MarkerReadError<std::io::Error>) -> Self {
        Error::MarkerRead(e)
    }
}
