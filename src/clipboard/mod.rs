mod error;

pub use error::ClipboardError;

use base64::Engine;
use std::io::Write;

/// Successful clipboard copy metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopySuccess {
    pub tool: String,
}

/// Clipboard abstraction used by picker code and tests.
pub trait Clipboard {
    fn copy(&self, text: &str) -> Result<CopySuccess, ClipboardError>;
}

/// Clipboard implementation that emits OSC 52 for Herdr to forward.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClipboard;

impl Clipboard for SystemClipboard {
    fn copy(&self, text: &str) -> Result<CopySuccess, ClipboardError> {
        write_osc52(std::io::stdout(), text.as_bytes())?;
        Ok(CopySuccess {
            tool: "osc52".to_string(),
        })
    }
}

/// Copies text through OSC 52 on stdout.
pub fn copy_to_system_clipboard(text: &str) -> Result<CopySuccess, ClipboardError> {
    SystemClipboard.copy(text)
}

fn write_osc52(mut writer: impl Write, bytes: &[u8]) -> Result<(), ClipboardError> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    write!(writer, "\x1b]52;c;{encoded}\x07").map_err(|err| ClipboardError::WriteFailed {
        message: err.to_string(),
    })?;
    writer.flush().map_err(|err| ClipboardError::WriteFailed {
        message: err.to_string(),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_osc52_uses_bel_terminated_clipboard_sequence() {
        let mut out = Vec::new();
        write_osc52(&mut out, b"hello").unwrap();
        assert_eq!(out, b"\x1b]52;c;aGVsbG8=\x07");
    }

    #[test]
    fn write_osc52_encodes_utf8_payload() {
        let mut out = Vec::new();
        write_osc52(&mut out, "https://example.com".as_bytes()).unwrap();
        assert_eq!(out, b"\x1b]52;c;aHR0cHM6Ly9leGFtcGxlLmNvbQ==\x07");
    }
}
