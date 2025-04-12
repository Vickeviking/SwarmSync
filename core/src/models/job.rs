use crate::enums::auth::HashAlgorithm;
use crate::enums::job::{FetchStyle, JobSchedule, OutputType};

#[derive(Debug, Clone)]
pub struct Checksum {
    checksum: Option<String>, // Optional SHA-256 or similar for file validation
    hash_algorithm: Option<HashAlgorithm>, // Consumer-specified hash algorithm for file verification
}

#[derive(Debug, Clone)]
pub enum ImageFormat {
    Tarball,        // Direct tarball download (.tar, .gz)
    DockerRegistry, // Pull using `docker pull`
}

#[derive(Debug, Clone)]
pub struct JobPayload {
    job_name: String,
    image_url: String,
    image_format: ImageFormat,
    docker_flags: Option<Vec<String>>, // e.g., ["--env", "FOO=bar"]
    output: OutputType,                // How to retrieve result
    fetch_style: FetchStyle,           // Pull or Push result
    schedule: JobSchedule,             // When to run
    checksum: Checksum,                // Optional file hash
    notes: Option<String>,             // Optional metadata
}
