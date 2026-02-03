# prom-cli

CLI for querying Prometheus/Mimir.

## Installation

```bash
cargo build --release
cp target/release/prom-cli ~/.local/bin/
```

## Configuration

Required environment variables:

```bash
export PROMETHEUS_URL="https://your-prometheus-url"
export PROMETHEUS_USER="username"
export PROMETHEUS_PASSWORD="password"
```

## Usage

```bash
# Instant query
prom-cli query 'up{job="prometheus"}'
prom-cli -H query 'rate(http_requests_total[5m])'  # human-readable

# Range query
prom-cli range 'up' --start 1h --end now --step 1m
prom-cli range 'rate(http_requests_total[5m])' --start 2024-01-01T00:00:00Z --end now --step 5m

# List metrics
prom-cli metrics
prom-cli metrics | grep http
```

### Options

| Option | Description |
|--------|-------------|
| `-H, --human` | Human-readable output with colors |

### Time formats

- `now` : current timestamp
- `1h`, `30m`, `2d` : relative duration (X time ago)
- `2024-01-01T00:00:00Z` : RFC3339
- `1704067200` : Unix timestamp
