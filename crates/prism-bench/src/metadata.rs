#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunMetadata {
    pub cpu_model: String,
    pub cores: String,
    pub ram_bytes: String,
    pub os: String,
    pub governor: String,
    pub affinity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub cpu_model: String,
    pub physical_cores: String,
    pub logical_cores: String,
    pub ram_bytes: String,
    pub os: String,
    pub kernel: String,
    pub runtime: String,
    pub compiler: String,
    pub governor: String,
    pub container_limits: String,
    pub affinity: String,
    pub commit: String,
    pub command: String,
    pub run_id: String,
}

impl Metadata {
    pub fn collect(config: &crate::cli::Config) -> Self {
        Self {
            cpu_model: config.metadata[0].clone(),
            physical_cores: "N/D".to_owned(),
            logical_cores: config.metadata[1].clone(),
            ram_bytes: config.metadata[2].clone(),
            os: config.metadata[3].clone(),
            kernel: "N/D".to_owned(),
            runtime: "rust".to_owned(),
            compiler: "rustc".to_owned(),
            governor: config.metadata[4].clone(),
            container_limits: "N/D".to_owned(),
            affinity: config.metadata[5].clone(),
            commit: "N/D".to_owned(),
            command: "N/D".to_owned(),
            run_id: "N/D".to_owned(),
        }
    }
}
