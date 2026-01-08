//! Error types for SceneSplit.

use std::path::PathBuf;
use thiserror::Error;

/// Supported video formats.
pub const SUPPORTED_FORMATS: &[&str] = &[
    "mp4", "avi", "mov", "mkv", "webm", "m4v", "flv", "wmv", "mpeg", "mpg",
];

/// Result type alias for SceneSplit operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during SceneSplit operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Video file not found.
    #[error("Video file not found: {0}")]
    VideoNotFound(PathBuf),

    /// Unsupported video format.
    #[error("Unsupported video format: '{extension}'. Supported: {}", SUPPORTED_FORMATS.join(", "))]
    UnsupportedFormat { path: PathBuf, extension: String },

    /// Error decoding video.
    #[error("Failed to decode video '{path}': {reason}")]
    VideoDecode { path: PathBuf, reason: String },

    /// Error with video capture.
    #[error("Video capture error: {0}")]
    #[allow(dead_code)]
    VideoCapture(String),

    /// Error computing embeddings.
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Error loading ONNX model.
    #[error("Model load error: {0}")]
    ModelLoad(String),

    /// Error writing output.
    #[error("Output error: {0}")]
    Output(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// OpenCV error.
    #[error("OpenCV error: {0}")]
    OpenCV(String),

    /// ONNX Runtime error.
    #[error("ONNX Runtime error: {0}")]
    Onnx(String),
}

impl From<opencv::Error> for Error {
    fn from(e: opencv::Error) -> Self {
        Error::OpenCV(e.message)
    }
}

impl From<ort::Error> for Error {
    fn from(e: ort::Error) -> Self {
        Error::Onnx(e.to_string())
    }
}
