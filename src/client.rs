use anyhow::{Context, Result};
use reqwest::blocking::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::env;
use std::time::Duration;

/// Trim a response body for inclusion in an error message — backend error pages can be
/// large, and the first line carries the signal.
fn snippet(body: &str) -> String {
    let body = body.trim();
    const MAX: usize = 300;
    if body.len() <= MAX {
        body.to_string()
    } else {
        format!("{}… ({} bytes)", &body[..MAX], body.len())
    }
}

#[derive(Deserialize, Debug)]
pub struct PrometheusResponse<T> {
    pub status: String,
    pub data: T,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(rename = "errorType", default)]
    pub error_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct QueryData {
    #[serde(rename = "resultType")]
    #[allow(dead_code)]
    pub result_type: String,
    pub result: Vec<QueryResult>,
}

#[derive(Deserialize, Debug)]
pub struct QueryResult {
    pub metric: serde_json::Value,
    #[serde(default)]
    pub value: Option<(f64, String)>,
    #[serde(default)]
    pub values: Option<Vec<(f64, String)>>,
}

pub struct PrometheusClient {
    client: Client,
    base_url: String,
    user: Option<String>,
    password: Option<String>,
}

impl PrometheusClient {
    pub fn new(timeout_secs: u64) -> Result<Self> {
        let base_url = env::var("PROMETHEUS_URL")
            .context("PROMETHEUS_URL environment variable not set")?;
        let user = env::var("PROMETHEUS_USER").ok();
        let password = env::var("PROMETHEUS_PASSWORD").ok();

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url,
            user,
            password,
        })
    }

    fn request(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);
        if let (Some(user), Some(pass)) = (&self.user, &self.password) {
            req = req.basic_auth(user, Some(pass));
        }
        req
    }

    /// Send the request and parse the Prometheus envelope. Reads the body as text first so a
    /// non-2xx response (e.g. the OVH promql-api returns HTTP 500 = aggregator OOM on a
    /// high-cardinality / `offset` query, or HTTP 422 = the 30 MB estimated-memory cap) is
    /// reported as a backend error with its code and body — not masked as a misleading
    /// "missing field `status`" JSON-parse failure from blindly deserializing the error page.
    fn fetch<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<PrometheusResponse<T>> {
        let resp = req.send().context("Failed to send request")?;
        let status = resp.status();
        let body = resp.text().context("Failed to read response body")?;
        if !status.is_success() {
            anyhow::bail!("backend returned HTTP {}: {}", status.as_u16(), snippet(&body));
        }
        serde_json::from_str(&body).with_context(|| {
            format!(
                "Failed to parse response (HTTP {}): {}",
                status.as_u16(),
                snippet(&body)
            )
        })
    }

    pub fn query(&self, promql: &str, at: Option<&str>) -> Result<QueryData> {
        let mut params = vec![("query", promql)];
        if let Some(time) = at {
            params.push(("time", time));
        }
        let resp: PrometheusResponse<QueryData> =
            self.fetch(self.request("/api/v1/query").query(&params))?;

        if resp.status != "success" {
            anyhow::bail!(
                "Query failed: {} - {}",
                resp.error_type.unwrap_or_default(),
                resp.error.unwrap_or_default()
            );
        }

        Ok(resp.data)
    }

    pub fn query_range(
        &self,
        promql: &str,
        start: &str,
        end: &str,
        step: &str,
    ) -> Result<QueryData> {
        let resp: PrometheusResponse<QueryData> = self.fetch(
            self.request("/api/v1/query_range")
                .query(&[("query", promql), ("start", start), ("end", end), ("step", step)]),
        )?;

        if resp.status != "success" {
            anyhow::bail!(
                "Query failed: {} - {}",
                resp.error_type.unwrap_or_default(),
                resp.error.unwrap_or_default()
            );
        }

        Ok(resp.data)
    }

    pub fn list_metrics(&self) -> Result<Vec<String>> {
        let resp: PrometheusResponse<Vec<String>> =
            self.fetch(self.request("/api/v1/label/__name__/values"))?;

        if resp.status != "success" {
            anyhow::bail!(
                "Failed to list metrics: {} - {}",
                resp.error_type.unwrap_or_default(),
                resp.error.unwrap_or_default()
            );
        }

        Ok(resp.data)
    }
}
