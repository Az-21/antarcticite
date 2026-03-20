use anyhow::{Context, Result};
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ExtensionMessage {
    ResolvedUrl(ResolvedUrlData),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResolvedUrlData {
    pub original_url: String,
    pub resolved_url: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum NativeMessage {
    Ack(AckData),
    Error(ErrorData),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AckData {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorData {
    pub status: String,
    pub message: String,
}

/// Reads a native messaging payload from standard input.
pub fn read_message() -> Result<Option<ExtensionMessage>> {
    let mut stdin = io::stdin();

    // Read the 32-bit length prefix
    let length = match stdin.read_u32::<NativeEndian>() {
        Ok(len) => len as usize,
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            // Extension disconnected
            return Ok(None);
        }
        Err(e) => return Err(e).context("Failed to read message length prefix"),
    };

    if length == 0 {
        return Ok(None);
    }

    // Read the JSON payload
    let mut buffer = vec![0; length];
    stdin
        .read_exact(&mut buffer)
        .context("Failed to read message payload")?;

    let json_str = String::from_utf8(buffer).context("Payload is not valid UTF-8")?;
    let message: ExtensionMessage = serde_json::from_str(&json_str)
        .with_context(|| format!("Failed to parse JSON: {}", json_str))?;

    Ok(Some(message))
}

/// Writes a native messaging payload to standard output.
pub fn write_message(message: &NativeMessage) -> Result<()> {
    let json_str = serde_json::to_string(message).context("Failed to serialize message")?;
    let buffer = json_str.as_bytes();

    let mut stdout = io::stdout();

    // Write the 32-bit length prefix
    stdout
        .write_u32::<NativeEndian>(buffer.len() as u32)
        .context("Failed to write length prefix")?;

    // Write the JSON payload
    stdout
        .write_all(buffer)
        .context("Failed to write payload")?;
    stdout.flush().context("Failed to flush stdout")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_ack() {
        let msg = NativeMessage::Ack(AckData {
            status: "success".to_string(),
            message: "URL routed to profile A".to_string(),
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(
            json,
            r#"{"type":"Ack","data":{"status":"success","message":"URL routed to profile A"}}"#
        );
    }

    #[test]
    fn test_deserialize_resolved_url() {
        let json = r#"{
          "type": "ResolvedUrl",
          "data": {
            "original_url": "https://protect-eu.mimecast.com/s/...",
            "resolved_url": "https://www.clientX.com/dashboard",
            "timestamp_ms": 1710950400000
          }
        }"#;

        let msg: ExtensionMessage = serde_json::from_str(json).unwrap();
        match msg {
            ExtensionMessage::ResolvedUrl(data) => {
                assert_eq!(data.original_url, "https://protect-eu.mimecast.com/s/...");
                assert_eq!(data.resolved_url, "https://www.clientX.com/dashboard");
                assert_eq!(data.timestamp_ms, 1710950400000);
            }
        }
    }
}
