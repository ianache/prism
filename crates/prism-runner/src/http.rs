use std::fmt;

pub const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_HEADER_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub content_type: Option<String>,
    pub authorization: Option<String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn bearer_token(&self) -> Option<&str> {
        self.authorization
            .as_deref()
            .and_then(|value| value.strip_prefix("Bearer "))
            .filter(|value| !value.is_empty() && !value.contains(char::is_whitespace))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    BadRequest,
    Unauthorized,
    NotFound,
    MethodNotAllowed,
    PayloadTooLarge,
    UnsupportedMediaType,
}

impl HttpError {
    pub fn status(&self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::NotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::PayloadTooLarge => 413,
            Self::UnsupportedMediaType => 415,
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BadRequest => "bad request",
            Self::Unauthorized => "authentication required",
            Self::NotFound => "not found",
            Self::MethodNotAllowed => "method not allowed",
            Self::PayloadTooLarge => "payload too large",
            Self::UnsupportedMediaType => "unsupported media type",
        })
    }
}

pub fn parse_request(raw: &[u8]) -> Result<Request, HttpError> {
    let Some(separator) = raw.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Err(HttpError::BadRequest);
    };
    let header_end = separator + 4;
    if header_end > MAX_HEADER_BYTES {
        return Err(HttpError::PayloadTooLarge);
    }
    let header_text = std::str::from_utf8(&raw[..separator]).map_err(|_| HttpError::BadRequest)?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or(HttpError::BadRequest)?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().ok_or(HttpError::BadRequest)?;
    let path = request_parts.next().ok_or(HttpError::BadRequest)?;
    let version = request_parts.next().ok_or(HttpError::BadRequest)?;
    if request_parts.next().is_some() || version != "HTTP/1.1" {
        return Err(HttpError::BadRequest);
    }

    let mut content_type = None;
    let mut authorization = None;
    let mut content_length = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return Err(HttpError::BadRequest);
        };
        let value = value.trim();
        match name.to_ascii_lowercase().as_str() {
            "content-type" => content_type = Some(value.to_owned()),
            "authorization" => authorization = Some(value.to_owned()),
            "content-length" => {
                if content_length.is_some() {
                    return Err(HttpError::BadRequest);
                }
                content_length = Some(value.parse::<usize>().map_err(|_| HttpError::BadRequest)?);
            }
            "transfer-encoding" => return Err(HttpError::BadRequest),
            _ => {}
        }
    }

    let body_length = content_length.ok_or(HttpError::BadRequest)?;
    if body_length > MAX_BODY_BYTES {
        return Err(HttpError::PayloadTooLarge);
    }
    if raw.len() - header_end != body_length {
        return Err(HttpError::BadRequest);
    }

    match (method, path) {
        ("GET", "/healthz") | ("GET", "/readyz") if body_length == 0 => {}
        ("POST", "/v1/process") => {
            if content_type.as_deref() != Some("application/json") {
                return Err(HttpError::UnsupportedMediaType);
            }
        }
        ("GET", _) | ("POST", _) => return Err(HttpError::NotFound),
        _ => return Err(HttpError::MethodNotAllowed),
    }

    Ok(Request {
        method: method.to_owned(),
        path: path.to_owned(),
        content_type: content_type.map(|value| value.to_owned()),
        authorization: authorization.map(|value| value.to_owned()),
        body: raw[header_end..].to_vec(),
    })
}

pub fn response_bytes(status: u16, body: &[u8]) -> Vec<u8> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}
