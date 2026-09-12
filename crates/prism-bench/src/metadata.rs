#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunMetadata {
    pub cpu_model: String,
    pub cores: String,
    pub ram_bytes: String,
    pub os: String,
    pub governor: String,
    pub affinity: String,
}
