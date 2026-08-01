use thiserror::Error;

/// User-facing failure while copying text to the clipboard.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClipboardError {
    #[error("failed to write OSC 52 clipboard sequence: {message}")]
    WriteFailed { message: String },
}
