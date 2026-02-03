use crate::client::PrometheusClient;
use crate::format::format_query_result;
use anyhow::Result;

pub fn run(client: &PrometheusClient, promql: &str, human_readable: bool) -> Result<()> {
    let data = client.query(promql)?;
    format_query_result(&data, human_readable);
    Ok(())
}
