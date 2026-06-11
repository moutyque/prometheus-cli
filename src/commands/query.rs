use crate::client::PrometheusClient;
use crate::commands::range::parse_time;
use crate::format::format_query_result_limited;
use anyhow::Result;

pub fn run(
    client: &PrometheusClient,
    promql: &str,
    at: Option<&str>,
    human_readable: bool,
    limit: Option<usize>,
) -> Result<()> {
    let at_ts = at.map(parse_time).transpose()?;
    let data = client.query(promql, at_ts.as_deref())?;
    format_query_result_limited(&data, human_readable, limit);
    Ok(())
}
