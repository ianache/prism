use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliError {
    MissingMetadata,
    MissingValue,
    UnknownFlag,
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingMetadata => "missing required metadata",
            Self::MissingValue => "missing flag value",
            Self::UnknownFlag => "unknown flag",
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
}

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
            "--levels" | "--scenario" | "--warmup" | "--samples" | "--repetitions" | "--output" => {
                None
            }
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
            _ => metadata[target.unwrap()] = Some(value),
        }
        index += 2;
    }
    if dataset.is_none() || metadata.iter().any(Option::is_none) {
        return Err(CliError::MissingMetadata);
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
    })
}
