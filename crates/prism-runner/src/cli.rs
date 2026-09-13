#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    B0,
    B1,
    B2,
}

impl Route {
    pub fn parse(value: &str) -> Result<Self, CliError> {
        match value {
            "b0" => Ok(Self::B0),
            "b1" => Ok(Self::B1),
            "b2" => Ok(Self::B2),
            _ => Err(CliError::UnknownRoute(value.to_owned())),
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::B0 => "b0",
            Self::B1 => "b1",
            Self::B2 => "b2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub route: Route,
    pub request_id_prefix: String,
    pub listen: Option<String>,
    pub workers: usize,
    pub connection_queue: usize,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub auth_token_file: Option<String>,
    pub shutdown_file: Option<String>,
    pub drain_timeout_ms: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    Help,
    MissingRoute,
    UnknownArgument(String),
    MissingValue(String),
    UnknownRoute(String),
    InvalidValue(String),
}

pub fn parse_args<I, S>(args: I) -> Result<Args, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut iter = args.into_iter().map(Into::into).skip(1);
    let mut route = None;
    let mut prefix = String::from("req");
    let mut listen = None;
    let mut workers = 1usize;
    let mut connection_queue = 0usize;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut auth_token_file = None;
    let mut shutdown_file = None;
    let mut drain_timeout_ms = 5_000usize;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err(CliError::Help),
            "--route" => {
                route = Some(Route::parse(
                    &iter
                        .next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                )?)
            }
            "--request-id-prefix" => {
                prefix = iter
                    .next()
                    .ok_or_else(|| CliError::MissingValue(arg.clone()))?
            }
            "--listen" => {
                listen = Some(
                    iter.next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                )
            }
            "--workers" => {
                workers = parse_bounded_number(
                    &arg,
                    &iter
                        .next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                )?;
            }
            "--connection-queue" => {
                connection_queue = parse_bounded_number(
                    &arg,
                    &iter
                        .next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                )?;
            }
            "--tls-cert" => {
                tls_cert = Some(
                    iter.next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                );
            }
            "--tls-key" => {
                tls_key = Some(
                    iter.next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                );
            }
            "--auth-token-file" => {
                auth_token_file = Some(
                    iter.next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                );
            }
            "--shutdown-file" => {
                shutdown_file = Some(
                    iter.next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                );
            }
            "--drain-timeout-ms" => {
                drain_timeout_ms = parse_timeout(
                    &arg,
                    &iter
                        .next()
                        .ok_or_else(|| CliError::MissingValue(arg.clone()))?,
                )?;
            }
            _ if arg.starts_with("--") => return Err(CliError::UnknownArgument(arg)),
            _ => return Err(CliError::UnknownArgument(arg)),
        }
    }
    Ok(Args {
        route: route.ok_or(CliError::MissingRoute)?,
        request_id_prefix: prefix,
        listen,
        workers,
        connection_queue,
        tls_cert,
        tls_key,
        auth_token_file,
        shutdown_file,
        drain_timeout_ms,
    })
}

fn parse_timeout(flag: &str, value: &str) -> Result<usize, CliError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| CliError::InvalidValue(flag.to_owned()))?;
    if parsed == 0 || parsed > 600_000 {
        return Err(CliError::InvalidValue(flag.to_owned()));
    }
    Ok(parsed)
}

fn parse_bounded_number(flag: &str, value: &str) -> Result<usize, CliError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| CliError::InvalidValue(flag.to_owned()))?;
    if parsed > 256 || (flag == "--workers" && parsed == 0) {
        return Err(CliError::InvalidValue(flag.to_owned()));
    }
    Ok(parsed)
}

pub fn usage() -> &'static str {
    "usage: prism-run --route <b0|b1|b2> [--request-id-prefix <prefix>] [--listen <host:port>] [--workers <n>] [--connection-queue <n>] [--tls-cert <certificate.pem> --tls-key <private-key.pem>] [--auth-token-file <token-file>] [--shutdown-file <path>] [--drain-timeout-ms <ms>]"
}
