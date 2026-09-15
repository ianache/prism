use std::io::{self, BufRead, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use prism_runner::{lifecycle, parse_args, security, transport, usage, CliError};

fn main() {
    let args = match parse_args(std::env::args()) {
        Ok(args) => args,
        Err(CliError::Help) => {
            println!("{}", usage());
            return;
        }
        Err(error) => {
            eprintln!("{}: {:?}", usage(), error);
            std::process::exit(2);
        }
    };
    if args.listen.is_none()
        && (args.tls_cert.is_some() || args.tls_key.is_some() || args.auth_token_file.is_some())
    {
        eprintln!("TLS and authentication options require --listen");
        std::process::exit(2);
    }
    let security = match security::load(
        args.tls_cert.as_deref(),
        args.tls_key.as_deref(),
        args.auth_token_file.as_deref(),
    ) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("security configuration invalid: {error}");
            std::process::exit(2);
        }
    };
    if args.shutdown_file.is_some() {
        eprintln!("STARTING");
    }
    if let Some(address) = args.listen.as_deref() {
        let address = match transport::parse_listen(address) {
            Ok(address) => address,
            Err(error) => {
                eprintln!("listen address invalid: {error}");
                std::process::exit(2);
            }
        };
        let listener = match TcpListener::bind(address) {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("listen failed: {error}");
                std::process::exit(2);
            }
        };
        let controller =
            lifecycle::Controller::new(Duration::from_millis(args.drain_timeout_ms as u64));
        let watcher = args
            .shutdown_file
            .as_ref()
            .map(|path| lifecycle::watch_file(PathBuf::from(path), controller.flag()));
        let result = transport::serve(
            listener,
            args.protocol,
            args.route,
            &args.request_id_prefix,
            args.workers,
            args.connection_queue,
            controller.flag(),
            controller.deadline,
            security,
        );
        controller.signal();
        if let Some(handle) = watcher {
            let _ = handle.join();
        }
        if let Err(error) = result {
            eprintln!("transport failed: {error}");
            std::process::exit(2);
        }
        return;
    }
    if args.workers != 1
        || args.connection_queue != 0
        || args.shutdown_file.is_some()
        || args.drain_timeout_ms != 5000
    {
        eprintln!("--workers, --connection-queue, --shutdown-file and --drain-timeout-ms require --listen");
        std::process::exit(2);
    }
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for (index, line) in stdin.lock().lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("input failed: {error}");
                std::process::exit(2);
            }
        };
        write_line(
            &mut stdout,
            &prism_runner::process_line(args.route, &args.request_id_prefix, index + 1, &line),
        );
    }
    if stdout.flush().is_err() {
        std::process::exit(2);
    }
}

fn write_line(stdout: &mut impl Write, value: &str) {
    if writeln!(stdout, "{value}")
        .and_then(|_| stdout.flush())
        .is_err()
    {
        std::process::exit(2);
    }
}
