use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;
use std::time::Duration;

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

    fn request(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);
        if let (Some(user), Some(pass)) = (&self.user, &self.password) {
            req = req.basic_auth(user, Some(pass));
        }
        req
    }

    pub fn query(&self, promql: &str, at: Option<&str>) -> Result<QueryData> {
        let mut params = vec![("query", promql)];
        if let Some(time) = at {
            params.push(("time", time));
        }
        let resp: PrometheusResponse<QueryData> = self
            .request("/api/v1/query")
            .query(&params)
            .send()
            .context("Failed to send request")?
            .json()
            .context("Failed to parse response")?;

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
        let resp: PrometheusResponse<QueryData> = self
            .request("/api/v1/query_range")
            .query(&[("query", promql), ("start", start), ("end", end), ("step", step)])
            .send()
            .context("Failed to send request")?
            .json()
            .context("Failed to parse response")?;

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
        let resp: PrometheusResponse<Vec<String>> = self
            .request("/api/v1/label/__name__/values")
            .send()
            .context("Failed to send request")?
            .json()
            .context("Failed to parse response")?;

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
