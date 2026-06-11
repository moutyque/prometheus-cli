use crate::client::PrometheusClient;
use crate::format::format_query_result_limited;
use anyhow::Result;
use chrono::Utc;

pub fn run(
    client: &PrometheusClient,
    promql: &str,
    start: &str,
    end: &str,
    step: &str,
    human_readable: bool,
    limit: Option<usize>,
) -> Result<()> {
    let start_ts = parse_time(start)?;
    let end_ts = parse_time(end)?;

    let data = client.query_range(promql, &start_ts, &end_ts, step)?;
    format_query_result_limited(&data, human_readable, limit);
    Ok(())
}

pub fn parse_time(input: &str) -> Result<String> {
    if input == "now" {
        return Ok(Utc::now().timestamp().to_string());
    }

    // Try parsing as duration (e.g., "1h", "30m")
    if let Ok(duration) = humantime::parse_duration(input) {
        let ts = Utc::now() - chrono::Duration::from_std(duration)?;
        return Ok(ts.timestamp().to_string());
    }

    // Try parsing as RFC3339
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(input) {
        return Ok(dt.timestamp().to_string());
    }

    // Try parsing as Unix timestamp
    if input.parse::<i64>().is_ok() {
        return Ok(input.to_string());
    }

    anyhow::bail!("Invalid time format: {}. Use 'now', duration (1h, 30m), RFC3339, or Unix timestamp", input)
}
