mod client;
mod commands;
mod format;

use anyhow::Result;
use clap::{Parser, Subcommand};
use client::PrometheusClient;

#[derive(Parser)]
#[command(name = "prom-cli")]
#[command(about = "Prometheus CLI for querying metrics")]
struct Cli {
    /// Human-readable output
    #[arg(short = 'H', long, global = true)]
    human: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute an instant PromQL query
    Query {
        /// PromQL query string
        promql: String,
    },
    /// Execute a range PromQL query
    Range {
        /// PromQL query string
        promql: String,
        /// Start time (e.g., "1h", "2024-01-01T00:00:00Z", Unix timestamp)
        #[arg(long)]
        start: String,
        /// End time (e.g., "now", "2024-01-01T01:00:00Z", Unix timestamp)
        #[arg(long)]
        end: String,
        /// Step interval (e.g., "1m", "15s")
        #[arg(long)]
        step: String,
    },
    /// List all available metrics
    Metrics,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = PrometheusClient::new()?;

    match cli.command {
        Commands::Query { promql } => {
            commands::query::run(&client, &promql, cli.human)?;
        }
        Commands::Range {
            promql,
            start,
            end,
            step,
        } => {
            commands::range::run(&client, &promql, &start, &end, &step, cli.human)?;
        }
        Commands::Metrics => {
            commands::metrics::run(&client)?;
        }
    }

    Ok(())
}
