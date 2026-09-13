use std::io::{self, BufRead, Write};
use std::net::TcpListener;

use prism_runner::{parse_args, transport, usage, CliError};

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
        if let Err(error) = transport::serve(listener, args.route, &args.request_id_prefix, 0) {
            eprintln!("transport failed: {error}");
            std::process::exit(2);
        }
        return;
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
