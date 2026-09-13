use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

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
    shutdown: &Arc<AtomicBool>,
    drain_deadline: &Arc<Mutex<Option<Instant>>>,
    read_timeout: Duration,
) -> io::Result<()> {
    stream.set_read_timeout(Some(read_timeout))?;
    stream.set_write_timeout(Some(read_timeout))?;
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
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                let expired = drain_deadline
                    .lock()
                    .expect("deadline lock poisoned")
                    .map(|deadline| Instant::now() >= deadline)
                    .unwrap_or(false);
                if shutdown.load(Ordering::Acquire) && expired {
                    return Ok(());
                }
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn capacity(workers: usize, queue: usize) -> usize {
    workers.saturating_add(queue)
}

struct Job {
    stream: TcpStream,
    permits: Arc<Mutex<usize>>,
    shutdown: Arc<AtomicBool>,
    drain_deadline: Arc<Mutex<Option<Instant>>>,
    read_timeout: Duration,
}

fn worker_loop(receiver: Arc<Mutex<mpsc::Receiver<Job>>>, route: Route, prefix: String) {
    loop {
        let job = match receiver.lock().expect("worker queue lock poisoned").recv() {
            Ok(job) => job,
            Err(_) => return,
        };
        if let Err(error) = handle_connection(
            job.stream,
            route,
            &prefix,
            &mut 0usize,
            &job.shutdown,
            &job.drain_deadline,
            job.read_timeout,
        ) {
            eprintln!("connection failed: {error}");
        }
        *job.permits.lock().expect("capacity lock poisoned") += 1;
    }
}

fn reject(mut stream: TcpStream, route: Route, code: &str, message: &str) -> io::Result<()> {
    write_line(
        &mut stream,
        &output::error("connection", route.as_str(), code, message),
    )
}

pub fn serve(
    listener: TcpListener,
    route: Route,
    prefix: &str,
    workers: usize,
    queue: usize,
    shutdown: Arc<AtomicBool>,
    drain_timeout: Duration,
) -> io::Result<()> {
    let address = listener.local_addr()?;
    eprintln!("READY {}", address);
    listener.set_nonblocking(true)?;
    let limit = capacity(workers, queue);
    let permits = Arc::new(Mutex::new(limit));
    let drain_deadline = Arc::new(Mutex::new(None));
    let (sender, receiver) = mpsc::sync_channel(limit.max(1));
    let receiver = Arc::new(Mutex::new(receiver));
    let read_timeout = Duration::from_millis(drain_timeout.as_millis().min(100).max(1) as u64);
    let mut handles = Vec::new();
    for _ in 0..workers {
        let receiver = Arc::clone(&receiver);
        let prefix = prefix.to_owned();
        handles.push(thread::spawn(move || worker_loop(receiver, route, prefix)));
    }
    let mut deadline = None;
    let mut draining = false;
    loop {
        if shutdown.load(Ordering::Acquire) && !draining {
            eprintln!("DRAINING");
            *drain_deadline.lock().expect("deadline lock poisoned") =
                Some(Instant::now() + drain_timeout);
            deadline = Some(Instant::now() + drain_timeout);
            draining = true;
        }
        if draining {
            if Instant::now() >= deadline.expect("drain deadline missing") {
                break;
            }
        }
        match listener.accept() {
            Ok((stream, _)) => {
                if draining || shutdown.load(Ordering::Acquire) {
                    reject(stream, route, "SERVER_DRAINING", "server is draining")?;
                    continue;
                }
                let admitted = {
                    let mut available = permits.lock().expect("capacity lock poisoned");
                    if *available == 0 {
                        false
                    } else {
                        *available -= 1;
                        true
                    }
                };
                if !admitted {
                    reject(
                        stream,
                        route,
                        "CAPACITY_EXCEEDED",
                        "connection capacity is full",
                    )?;
                    continue;
                }
                let job = Job {
                    stream,
                    permits: Arc::clone(&permits),
                    shutdown: Arc::clone(&shutdown),
                    drain_deadline: Arc::clone(&drain_deadline),
                    read_timeout,
                };
                match sender.try_send(job) {
                    Ok(()) => {}
                    Err(mpsc::TrySendError::Full(job)) => {
                        *job.permits.lock().expect("capacity lock poisoned") += 1;
                        reject(
                            job.stream,
                            route,
                            "CAPACITY_EXCEEDED",
                            "connection capacity is full",
                        )?;
                    }
                    Err(mpsc::TrySendError::Disconnected(_)) => {
                        return Err(io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "worker pool stopped",
                        ))
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => return Err(error),
        }
    }
    drop(sender);
    let deadline = std::time::Instant::now() + drain_timeout;
    for handle in handles {
        if std::time::Instant::now() < deadline {
            let _ = handle.join();
        }
    }
    eprintln!("STOPPED");
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
    #[test]
    fn capacity_is_active_workers_plus_queue() {
        assert_eq!(capacity(2, 3), 5);
        assert_eq!(capacity(1, 0), 1);
    }
}
