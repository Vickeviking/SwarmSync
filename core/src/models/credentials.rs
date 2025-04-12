use crate::enums::auth::HashAlgorithm;
use chrono::{NaiveDateTime, Timelike};

//needed when user want to have their data pushed to them, using ssh key
#[derive(Debug, Clone)]
enum RetryIntervalType {
    Linear,
    Logarithmic,
    Exponential,
}

#[derive(Debug, Clone)]
pub struct PushCredentials {
    ssh_address: String,                              //address of consumers computer
    ssh_user: String,                                 //user name of user on computer
    ssh_key: Option<String>,                          // consumer public ssh key
    file_path: String,                                //where consumer would like result
    max_retries: u32, //max retries before storing with fetchstyle 'Pull' instead
    current_try: u32, //current try
    retry_interval_secs: u64, //current interval
    max_retry_interval_secs: u64, //max interval secs so we dont wait for years
    interval_increase_backof_func: RetryIntervalType, // backof function limited by 'max_retry_interval_sec'
    next_push_attempt_time: Option<NaiveDateTime>,    // timestamp for next try
    use_checksum: bool, // specify if you want to recieve a checksum of the hashed contents, to hash yourself and match the values.
    hash_algorithm: Option<HashAlgorithm>, // specify hash function (e.g., SHA-256)
}
