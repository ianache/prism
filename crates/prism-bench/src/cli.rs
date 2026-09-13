use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliError {
    MissingMetadata,
    MissingValue,
    UnknownFlag,
    InvalidScenario,
    InvalidLevel,
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingMetadata => "missing required metadata",
            Self::MissingValue => "missing flag value",
            Self::UnknownFlag => "unknown flag",
            Self::InvalidScenario => "invalid scenario",
            Self::InvalidLevel => "invalid level",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub dataset: String,
    pub metadata: [String; 6],
    pub levels: String,
    pub scenario: String,
    pub warmup: usize,
    pub samples: usize,
    pub repetitions: usize,
    pub output: String,
    pub command: String,
    pub warmup_frames: usize,
    pub convergence_window: usize,
    pub convergence_threshold_percent: u32,
    pub max_warmup_frames: usize,
    pub measured_frames: usize,
    pub concurrency: usize,
    pub concurrencies: Vec<usize>,
}

pub type CliConfig = Config;

pub fn parse_args<I, S>(args: I) -> Result<Config, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let values = args
        .into_iter()
        .map(|value| value.as_ref().to_owned())
        .collect::<Vec<_>>();
    let mut dataset = None;
    let mut metadata = [None, None, None, None, None, None];
    let mut levels = "b0,b1".to_owned();
    let mut scenario = "S1".to_owned();
    let mut warmup = 100usize;
    let mut samples = 100usize;
    let mut repetitions = 1usize;
    let mut output = "results/raw/s1.jsonl".to_owned();
    let mut concurrencies = vec![1usize];
    let mut index = 1;
    while index < values.len() {
        let flag = values[index].as_str();
        let target = match flag {
            "--dataset" => None,
            "--cpu-model" => Some(0),
            "--cores" => Some(1),
            "--ram-bytes" => Some(2),
            "--os" => Some(3),
            "--governor" => Some(4),
            "--affinity" => Some(5),
            "--levels" | "--scenario" | "--warmup" | "--samples" | "--repetitions" | "--output"
            | "--concurrency" => None,
            _ => return Err(CliError::UnknownFlag),
        };
        if index + 1 >= values.len() {
            return Err(CliError::MissingValue);
        }
        let value = values[index + 1].clone();
        match flag {
            "--dataset" => dataset = Some(value),
            "--levels" => levels = value,
            "--scenario" => scenario = value,
            "--warmup" => warmup = value.parse().map_err(|_| CliError::MissingValue)?,
            "--samples" => samples = value.parse().map_err(|_| CliError::MissingValue)?,
            "--repetitions" => repetitions = value.parse().map_err(|_| CliError::MissingValue)?,
            "--output" => output = value,
            "--concurrency" => {
                concurrencies = value
                    .split(',')
                    .map(|item| item.parse().map_err(|_| CliError::MissingValue))
                    .collect::<Result<Vec<usize>, CliError>>()?;
            }
            _ => metadata[target.unwrap()] = Some(value),
        }
        index += 2;
    }
    if dataset.is_none() || metadata.iter().any(Option::is_none) {
        return Err(CliError::MissingMetadata);
    }
    if scenario != "S1" && scenario != "S2" && scenario != "S3" && scenario != "S4" {
        return Err(CliError::InvalidScenario);
    }
    if levels.split(',').any(|level| !matches!(level, "b0" | "b1" | "b2")) {
        return Err(CliError::InvalidLevel);
    }
    if levels.split(',').any(|level| level == "b2") && !levels.split(',').any(|level| level == "b1") {
        return Err(CliError::MissingValue);
    }
    if scenario == "S2" && !(levels.split(',').any(|level| level == "b0")
        && levels.split(',').any(|level| level == "b1")
        && levels.split(',').any(|level| level == "b2")) {
        return Err(CliError::MissingValue);
    }
    if warmup == 0 || samples == 0 || repetitions == 0 || concurrencies.is_empty() {
        return Err(CliError::MissingValue);
    }
    if warmup < 10_000 || samples < 10_000 || repetitions != 5 {
        return Err(CliError::MissingValue);
    }
    let allowed_concurrencies = [1, 2, 4, 8, 16, 32, 64];
    if concurrencies.iter().any(|value| !allowed_concurrencies.contains(value))
        || concurrencies.iter().any(|value| concurrencies.iter().filter(|item| *item == value).count() > 1)
    {
        return Err(CliError::MissingValue);
    }
    if scenario != "S3" && concurrencies != [1] {
        return Err(CliError::MissingValue);
    }
    if scenario == "S3"
        && (levels.split(',').collect::<std::collections::BTreeSet<_>>()
            != ["b0", "b1", "b2"].into_iter().collect())
    {
        return Err(CliError::MissingValue);
    }
    if scenario == "S3" && (samples != 100_000 || concurrencies != allowed_concurrencies) {
        return Err(CliError::MissingValue);
    }
    if scenario == "S4"
        && (samples != 100_000
            || concurrencies != [1]
            || levels.split(',').collect::<std::collections::BTreeSet<_>>()
                != ["b0", "b1", "b2"].into_iter().collect())
    {
        return Err(CliError::MissingValue);
    }
    Ok(Config {
        dataset: dataset.unwrap(),
        metadata: metadata.map(Option::unwrap),
        levels,
        scenario,
        warmup,
        samples,
        repetitions,
        output,
        command: values.join(" "),
        warmup_frames: warmup,
        convergence_window: 1_000,
        convergence_threshold_percent: 5,
        max_warmup_frames: 1_000_000,
        measured_frames: samples,
        concurrency: concurrencies[0],
        concurrencies,
    })
}
