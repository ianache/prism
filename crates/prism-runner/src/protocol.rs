#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub request_id: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    pub code: &'static str,
    pub message: String,
}

fn string_field(input: &str, key: &str) -> Result<String, ProtocolError> {
    let needle = format!("\"{}\"", key);
    let start = input.find(&needle).ok_or_else(|| ProtocolError {
        code: "MISSING_FIELD",
        message: format!("missing field {key}"),
    })?;
    let rest = &input[start + needle.len()..];
    let colon = rest.find(':').ok_or(ProtocolError {
        code: "MALFORMED_JSON",
        message: "missing colon".into(),
    })?;
    let value = rest[colon + 1..].trim_start();
    if !value.starts_with('"') {
        return Err(ProtocolError {
            code: "INVALID_FIELD",
            message: format!("field {key} must be a string"),
        });
    }
    let mut escaped = false;
    let mut result = String::new();
    for ch in value[1..].chars() {
        if escaped {
            match ch {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                _ => {
                    return Err(ProtocolError {
                        code: "INVALID_ESCAPE",
                        message: format!("invalid escape in {key}"),
                    })
                }
            };
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Ok(result);
        } else {
            result.push(ch);
        }
    }
    Err(ProtocolError {
        code: "MALFORMED_JSON",
        message: format!("unterminated string {key}"),
    })
}

fn decode_hex(value: &str) -> Result<Vec<u8>, ProtocolError> {
    if value.is_empty() {
        return Err(ProtocolError {
            code: "EMPTY_PAYLOAD",
            message: "payload_hex is empty".into(),
        });
    }
    if value.len() % 2 != 0 {
        return Err(ProtocolError {
            code: "INVALID_HEX",
            message: "payload_hex must have even length".into(),
        });
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let chars = value.as_bytes();
    for pair in chars.chunks_exact(2) {
        let high = (pair[0] as char).to_digit(16);
        let low = (pair[1] as char).to_digit(16);
        match (high, low) {
            (Some(h), Some(l)) => bytes.push(((h << 4) | l) as u8),
            _ => {
                return Err(ProtocolError {
                    code: "INVALID_HEX",
                    message: "payload_hex contains a non-hex character".into(),
                })
            }
        }
    }
    Ok(bytes)
}

pub fn parse_line(line: &str) -> Result<Envelope, ProtocolError> {
    if line.trim().is_empty() {
        return Err(ProtocolError {
            code: "BLANK_LINE",
            message: "blank input line".into(),
        });
    }
    let trimmed = line.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(ProtocolError {
            code: "MALFORMED_JSON",
            message: "envelope must be a JSON object".into(),
        });
    }
    let request_id = string_field(trimmed, "request_id")?;
    let payload_hex = string_field(trimmed, "payload_hex")?;
    Ok(Envelope {
        request_id,
        payload: decode_hex(&payload_hex)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_envelope_and_hex() {
        assert_eq!(
            parse_line(r#"{"request_id":"a","payload_hex":"005aff"}"#)
                .unwrap()
                .payload,
            vec![0, 0x5a, 0xff]
        );
    }
    #[test]
    fn rejects_blank_and_bad_hex() {
        assert_eq!(parse_line(" ").unwrap_err().code, "BLANK_LINE");
        assert_eq!(
            parse_line(r#"{"request_id":"a","payload_hex":"0"}"#)
                .unwrap_err()
                .code,
            "INVALID_HEX"
        );
    }
    #[test]
    fn unescapes_request_id() {
        assert_eq!(
            parse_line(r#"{"request_id":"a\"b","payload_hex":"ff"}"#)
                .unwrap()
                .request_id,
            "a\"b"
        );
    }
}
