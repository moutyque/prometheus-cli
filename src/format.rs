use crate::client::QueryData;
use chrono::{DateTime, Utc};
use colored::Colorize;

pub fn format_query_result(data: &QueryData, human_readable: bool) {
    for result in &data.result {
        format_metric_header(&result.metric, human_readable);

        if let Some((ts, val)) = &result.value {
            format_value(*ts, val, human_readable);
        }

        if let Some(values) = &result.values {
            for (ts, val) in values {
                format_value(*ts, val, human_readable);
            }
        }
    }
}

fn format_metric_header(metric: &serde_json::Value, human_readable: bool) {
    if human_readable {
        if let Some(obj) = metric.as_object() {
            let name = obj.get("__name__").and_then(|v| v.as_str()).unwrap_or("");
            let labels: Vec<String> = obj
                .iter()
                .filter(|(k, _)| *k != "__name__")
                .map(|(k, v)| format!("{}={}", k.cyan(), v.to_string().yellow()))
                .collect();

            if labels.is_empty() {
                println!("{}", name.green().bold());
            } else {
                println!("{}{{{}}}", name.green().bold(), labels.join(", "));
            }
        }
    } else {
        println!("{}", metric);
    }
}

fn format_value(timestamp: f64, value: &str, human_readable: bool) {
    if human_readable {
        let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| timestamp.to_string());
        println!("  {} @ {}", value.bright_white(), dt.dimmed());
    } else {
        println!("{}\t{}", timestamp, value);
    }
}

pub fn format_metrics(metrics: &[String]) {
    for metric in metrics {
        println!("{}", metric);
    }
}
