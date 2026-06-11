use crate::client::{QueryData, QueryResult};
use chrono::{DateTime, Utc};
use colored::Colorize;

/* With a limit, only the N LARGEST series (by value; last sample for ranges) are
   printed, followed by a "[N of M series]" trailer ("(empty result)" when there are
   none) — a bounded, deterministic summary when a query returns thousands of series.
   Truncating in server order would drop an arbitrary subset. Without --limit, server
   order is preserved. */
pub fn format_query_result_limited(data: &QueryData, human_readable: bool, limit: Option<usize>) {
    let total = data.result.len();
    let shown = limit.map(|l| l.min(total)).unwrap_or(total);

    let mut results: Vec<&QueryResult> = data.result.iter().collect();
    if limit.is_some() {
        results.sort_by(|a, b| {
            sample_value(b)
                .partial_cmp(&sample_value(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    for result in results.into_iter().take(shown) {
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

    if limit.is_some() {
        if total == 0 {
            println!("(empty result)");
        } else {
            println!("[{} of {} series]", shown, total);
        }
    }
}

fn sample_value(r: &QueryResult) -> f64 {
    r.value
        .as_ref()
        .map(|(_, v)| v)
        .or_else(|| r.values.as_ref().and_then(|vs| vs.last()).map(|(_, v)| v))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(f64::NEG_INFINITY)
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
