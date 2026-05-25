# Analytics MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-analytics.svg)](https://crates.io/crates/mcp-analytics)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Governed analytics platform for AI agents — metrics, dashboards, funnels, cohorts, anomaly detection, forecasting, and policy enforcement. 28 tools with full audit trail.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-analytics/main/docs/assets/architecture.svg" alt="MCP Analytics Architecture" width="850"/>
</p>

## Tools (28)

### Discovery (5)

| Tool | Purpose |
|------|---------|
| `list_data_sources` | List connected data sources (warehouse, product DB, event stream) |
| `list_datasets` | List available datasets/tables with row counts |
| `describe_dataset` | Schema, columns, types, PII flags, freshness |
| `list_metrics` | List defined metrics with owners and certification |
| `get_metric_definition` | Full metric: formula, aggregation, dimensions |

### Querying (5)

| Tool | Purpose |
|------|---------|
| `query_metric` | Query metric over time range → time-series data |
| `breakdown_metric` | Break metric by dimension (country, plan, device) |
| `compare_metric` | Compare metric across two periods |
| `query_events` | Query raw events with filters |
| `query_report` | Run a saved/named report |

### Analysis (6)

| Tool | Purpose |
|------|---------|
| `analyze_funnel` | Conversion rates through funnel steps |
| `analyze_cohort` | Weekly/monthly cohort retention |
| `detect_anomalies` | Find anomalies in recent metric data |
| `forecast_metric` | Forecast metric N days forward with confidence |
| `explain_change` | Dimension attribution for metric changes |
| `generate_insight_summary` | AI-generated highlights + recommendations |

### Dashboard Building (5)

| Tool | Purpose |
|------|---------|
| `list_dashboards` | List all dashboards |
| `get_dashboard` | Get full dashboard with widgets |
| `summarize_dashboard` | Key takeaways from a dashboard |
| `create_dashboard` | Create a new dashboard |
| `add_widget` | Add chart/number/table/funnel widget |
| `publish_dashboard` | Publish dashboard to team |

### Governance (5)

| Tool | Purpose |
|------|---------|
| `validate_analytics_policy` | Check if action is allowed by policy |
| `check_export_risk` | Assess PII/row-count risk of export |
| `request_data_access` | Request approval for restricted data |
| `get_query_audit_trail` | View audit log of all queries |
| `get_segments` / `query_segment` | List and query user segments |

## Installation

```bash
cargo install mcp-analytics
```

## Configuration

No configuration required — starts with seeded demo data:
- 3 data sources, 4 datasets, 8 metrics
- 5 user segments, 2 funnels, 1 pre-built dashboard
- Policy enforcement active by default

### Policy Boundaries (enforced)

| Policy | Default |
|--------|---------|
| `raw_sql_allowed` | `false` |
| `pii_column_access` | `denied` |
| `row_limit` | `10,000` |
| `customer_level_export` | `requires_approval` |
| `employee_level_export` | `denied` |
| `financial_metric_access` | `approved_only` |
| `external_sharing_allowed` | `false` |
| `metric_certification_required` | `true` |

## Client Configuration

### Claude Desktop / Kiro / Cursor

```json
{
  "mcpServers": {
    "analytics": {
      "command": "mcp-analytics",
      "args": []
    }
  }
}
```

## End-to-End Example: Building a Revenue Dashboard

```
Agent: "Create a revenue dashboard for the exec team"

1. list_metrics → finds MRR, ARPU, Churn Rate
2. validate_analytics_policy(action="financial_access") → checks permission
3. create_dashboard(name="Revenue Dashboard")
4. query_metric(metric_id="m-mrr", days=30) → gets time-series
5. add_widget(dashboard_id, type="line_chart", title="MRR Trend", metric_id="m-mrr")
6. breakdown_metric(metric_id="m-mrr", dimension="plan") → by plan tier
7. add_widget(dashboard_id, type="bar_chart", title="MRR by Plan")
8. forecast_metric(metric_id="m-mrr", horizon_days=14)
9. add_widget(dashboard_id, type="line_chart", title="MRR Forecast")
10. detect_anomalies(id="m-churn") → finds spike
11. publish_dashboard(id=dashboard_id) → visible to team
```

## Governance Model

The analytics server enforces policy at every step:

- **Pre-query validation** — `validate_analytics_policy` checks before data access
- **Export risk assessment** — `check_export_risk` flags PII and large exports
- **Approval workflow** — `request_data_access` for restricted resources
- **Full audit trail** — every query logged with timestamp, params, result
- **No raw SQL** — prevents injection and uncontrolled data access

## MCP Server Manifest

```toml
server_id = "mcp_analytics"
display_name = "Analytics"
version = "1.0.0"
domain = "business-systems"
risk_level = "medium"
writes_allowed = "gated"
```

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)
