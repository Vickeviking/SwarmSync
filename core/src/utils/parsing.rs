use chrono::NaiveDateTime;

pub fn parse_naive_datetime(input: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S%.f"))
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|e| format!("Invalid date-time format: {}", e))
}
