use std::io::{self, BufRead, Write};

use prism_runner::{dispatch, output, parse_args, protocol, usage, CliError};

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
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for (index, line) in stdin.lock().lines().enumerate() {
        let fallback_id = format!("{}-{}", args.request_id_prefix, index + 1);
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("input failed: {error}");
                std::process::exit(2);
            }
        };
        let envelope = match protocol::parse_line(&line) {
            Ok(envelope) => envelope,
            Err(error) => {
                write_line(
                    &mut stdout,
                    &output::error(
                        &fallback_id,
                        args.route.as_str(),
                        error.code,
                        &error.message,
                    ),
                );
                continue;
            }
        };
        let request_id = if envelope.request_id.is_empty() {
            fallback_id
        } else {
            format!("{}-{}", args.request_id_prefix, envelope.request_id)
        };
        let result = match dispatch::run(args.route, &envelope.payload) {
            Ok((outcome, stats)) => {
                output::success(&request_id, args.route.as_str(), &outcome, &stats)
            }
            Err(message) => output::error(
                &request_id,
                args.route.as_str(),
                "RUNTIME_FAILURE",
                &message,
            ),
        };
        write_line(&mut stdout, &result);
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
