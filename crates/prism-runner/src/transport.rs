use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

use crate::{output, process_line, Route};

pub const MAX_LINE_BYTES: usize = 64 * 1024;

pub fn parse_listen(value: &str) -> io::Result<SocketAddr> {
    value.to_socket_addrs()?.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "listen address resolved to no address",
        )
    })
}

fn read_bounded(reader: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    let mut terminated = false;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            break;
        }
        if let Some(position) = available.iter().position(|byte| *byte == b'\n') {
            bytes.extend_from_slice(&available[..position]);
            reader.consume(position + 1);
            terminated = true;
            break;
        }
        bytes.extend_from_slice(available);
        let length = available.len();
        reader.consume(length);
        if bytes.len() > MAX_LINE_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "LINE_TOO_LARGE"));
        }
    }
    if bytes.is_empty() && !terminated {
        return Ok(None);
    }
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    if bytes.len() > MAX_LINE_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "LINE_TOO_LARGE"));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "input is not UTF-8"))
}

fn write_line(stream: &mut TcpStream, value: &str) -> io::Result<()> {
    stream.write_all(value.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn handle_connection(
    stream: TcpStream,
    route: Route,
    prefix: &str,
    sequence: &mut usize,
) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut stream = stream;
    loop {
        match read_bounded(&mut reader) {
            Ok(Some(line)) => {
                *sequence += 1;
                write_line(&mut stream, &process_line(route, prefix, *sequence, &line))?;
            }
            Ok(None) => return Ok(()),
            Err(error)
                if error.kind() == io::ErrorKind::InvalidData
                    && error.to_string() == "LINE_TOO_LARGE" =>
            {
                *sequence += 1;
                write_line(
                    &mut stream,
                    &output::error(
                        &format!("{}-{}", prefix, *sequence),
                        route.as_str(),
                        "LINE_TOO_LARGE",
                        "input line exceeds 64 KiB",
                    ),
                )?;
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn serve(
    listener: TcpListener,
    route: Route,
    prefix: &str,
    mut sequence: usize,
) -> io::Result<()> {
    eprintln!("LISTENING {}", listener.local_addr()?);
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                handle_connection(stream, route, prefix, &mut sequence)?;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_ephemeral_loopback_address() {
        assert_eq!(
            parse_listen("127.0.0.1:0").unwrap().ip().to_string(),
            "127.0.0.1"
        );
    }
    #[test]
    fn rejects_invalid_address() {
        assert!(parse_listen("not-an-address").is_err());
    }
}
