use crate::models::credentials::PushCredentials;

#[derive(Debug, Clone)]
pub enum FetchStyle {
    Pull,                  // Consumer will fetch results from Core Archive
    Push(PushCredentials), // Core will push results to consumer using SSH
}

#[derive(Debug, Clone)]
pub enum OutputType {
    Stdout,             // Capture standard output from container
    Files(Vec<String>), // Relative paths inside container (mounted volume or copied after run)
}

#[derive(Debug, Clone)]
pub enum JobSchedule {
    Once,
    Cron(String), // Cron expression, e.g., "0 14 * * 5" for every Friday at 14:00
}

#[derive(Debug, Clone)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed(String),
}
