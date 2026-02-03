use crate::client::PrometheusClient;
use crate::format::format_metrics;
use anyhow::Result;

pub fn run(client: &PrometheusClient) -> Result<()> {
    let metrics = client.list_metrics()?;
    format_metrics(&metrics);
    Ok(())
}
