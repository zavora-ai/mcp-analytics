use crate::api::AnalyticsApi;
use crate::domain::*;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};

// --- Input types ---

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueryMetricInput {
    pub metric_id: String,
    pub days: Option<u32>,
    pub granularity: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BreakdownInput {
    pub metric_id: String,
    pub dimension: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompareInput {
    pub metric_id: String,
    pub period_a: String,
    pub period_b: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EventsInput {
    pub event_name: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FunnelInput { pub funnel_name: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ForecastInput {
    pub metric_id: String,
    pub horizon_days: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateDashboardInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddWidgetInput {
    pub dashboard_id: String,
    pub widget_type: String,
    pub title: String,
    pub metric_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PolicyCheckInput {
    pub action: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExportRiskInput {
    pub dataset_id: String,
    pub row_count: Option<u64>,
    pub includes_pii: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DataAccessInput {
    pub resource: String,
    pub reason: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SegmentQueryInput { pub segment_id: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReportInput { pub report_name: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplainInput {
    pub metric_id: String,
    pub date: Option<String>,
}

// --- Server ---

#[derive(Clone)]
pub struct AnalyticsServer {
    pub api: AnalyticsApi,
}

#[tool_router(server_handler)]
impl AnalyticsServer {
    // === Discovery (5) ===

    #[tool(description = "List all connected data sources")]
    async fn list_data_sources(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        serde_json::to_string_pretty(&self.api.data_sources).unwrap()
    }

    #[tool(description = "List all available datasets/tables")]
    async fn list_datasets(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let summary: Vec<serde_json::Value> = self.api.datasets.iter().map(|d| {
            serde_json::json!({ "id": d.id, "name": d.name, "source_id": d.source_id, "row_count": d.row_count, "freshness": d.freshness })
        }).collect();
        serde_json::to_string_pretty(&summary).unwrap()
    }

    #[tool(description = "Describe a dataset — schema, row count, freshness, column types")]
    async fn describe_dataset(&self, Parameters(input): Parameters<IdInput>) -> String {
        match self.api.datasets.iter().find(|d| d.id == input.id) {
            Some(d) => serde_json::to_string_pretty(d).unwrap(),
            None => format!("Dataset {} not found", input.id),
        }
    }

    #[tool(description = "List all defined metrics with owners and certification status")]
    async fn list_metrics(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let summary: Vec<serde_json::Value> = self.api.metrics.iter().map(|m| {
            serde_json::json!({ "id": m.id, "name": m.name, "aggregation": m.aggregation, "owner": m.owner, "certified": m.certified })
        }).collect();
        serde_json::to_string_pretty(&summary).unwrap()
    }

    #[tool(description = "Get full metric definition — formula, aggregation, dimensions, dataset")]
    async fn get_metric_definition(&self, Parameters(input): Parameters<IdInput>) -> String {
        match self.api.metrics.iter().find(|m| m.id == input.id) {
            Some(m) => serde_json::to_string_pretty(m).unwrap(),
            None => format!("Metric {} not found", input.id),
        }
    }

    // === Querying (5) ===

    #[tool(description = "Query a metric over a time range — returns time-series data")]
    async fn query_metric(&self, Parameters(input): Parameters<QueryMetricInput>) -> String {
        let days = input.days.unwrap_or(30);
        let gran = input.granularity.as_deref().unwrap_or("daily");
        let data = self.api.generate_time_series(&input.metric_id, days, gran);
        let total: f64 = data.iter().map(|p| p.value).sum();
        let name = self.api.metrics.iter().find(|m| m.id == input.metric_id).map(|m| m.name.clone()).unwrap_or(input.metric_id.clone());
        let result = MetricResult { metric_name: name, granularity: gran.into(), start_date: data.first().map(|p| p.date.clone()).unwrap_or_default(), end_date: data.last().map(|p| p.date.clone()).unwrap_or_default(), data, total: (total * 100.0).round() / 100.0 };
        self.api.log_audit("query_metric", serde_json::json!({"metric_id": input.metric_id}), "allowed").await;
        serde_json::to_string_pretty(&result).unwrap()
    }

    #[tool(description = "Break down a metric by dimension (country, plan, device, source)")]
    async fn breakdown_metric(&self, Parameters(input): Parameters<BreakdownInput>) -> String {
        let segments = self.api.generate_breakdown(&input.metric_id, &input.dimension);
        let name = self.api.metrics.iter().find(|m| m.id == input.metric_id).map(|m| m.name.clone()).unwrap_or(input.metric_id.clone());
        let result = BreakdownResult { metric_name: name, dimension: input.dimension, segments };
        serde_json::to_string_pretty(&result).unwrap()
    }

    #[tool(description = "Compare a metric across two time periods")]
    async fn compare_metric(&self, Parameters(input): Parameters<CompareInput>) -> String {
        let a = self.api.generate_time_series(&input.metric_id, 7, "daily");
        let b = self.api.generate_time_series(&input.metric_id, 7, "daily");
        let avg_a: f64 = a.iter().map(|p| p.value).sum::<f64>() / a.len() as f64;
        let avg_b: f64 = b.iter().map(|p| p.value).sum::<f64>() / b.len() as f64;
        let change_pct = ((avg_b - avg_a) / avg_a * 100.0 * 100.0).round() / 100.0;
        serde_json::to_string_pretty(&serde_json::json!({
            "metric_id": input.metric_id,
            "period_a": input.period_a, "period_b": input.period_b,
            "avg_a": (avg_a * 100.0).round() / 100.0,
            "avg_b": (avg_b * 100.0).round() / 100.0,
            "change_percent": change_pct,
            "direction": if change_pct > 0.0 { "up" } else { "down" }
        })).unwrap()
    }

    #[tool(description = "Query raw events with optional event name filter")]
    async fn query_events(&self, Parameters(input): Parameters<EventsInput>) -> String {
        let limit = input.limit.unwrap_or(10) as usize;
        let events: Vec<Event> = (0..limit).map(|i| {
            let name = input.event_name.clone().unwrap_or_else(|| ["page_view", "signup", "purchase", "feature_used"][i % 4].into());
            Event {
                timestamp: chrono::Utc::now().to_rfc3339(),
                event_name: name,
                user_id: format!("user-{}", 1000 + i),
                properties: serde_json::json!({"page": "/home", "source": "organic"}),
            }
        }).collect();
        serde_json::to_string_pretty(&events).unwrap()
    }

    #[tool(description = "Run a saved/named report")]
    async fn query_report(&self, Parameters(input): Parameters<ReportInput>) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "report": input.report_name,
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "summary": format!("Report '{}' executed successfully. Contains aggregated metrics for the last 30 days.", input.report_name),
            "metrics_included": ["DAU", "Signups", "Revenue", "Churn Rate"]
        })).unwrap()
    }

    // === Analysis (6) ===

    #[tool(description = "Analyze funnel conversion rates through steps")]
    async fn analyze_funnel(&self, Parameters(input): Parameters<FunnelInput>) -> String {
        match self.api.funnels.iter().find(|f| f.0 == input.funnel_name) {
            Some((name, steps)) => {
                let funnel_steps: Vec<FunnelStep> = steps.iter().enumerate().map(|(i, (sname, count))| {
                    let conv = if i == 0 { 100.0 } else { *count as f64 / steps[i-1].1 as f64 * 100.0 };
                    FunnelStep { name: sname.to_string(), count: *count, conversion_from_prev: (conv * 10.0).round() / 10.0 }
                }).collect();
                let overall = *steps.last().map(|(_, c)| c).unwrap_or(&0) as f64 / steps[0].1 as f64 * 100.0;
                let result = FunnelResult { name: name.to_string(), steps: funnel_steps, overall_conversion: (overall * 10.0).round() / 10.0 };
                serde_json::to_string_pretty(&result).unwrap()
            }
            None => format!("Funnel '{}' not found. Available: {:?}", input.funnel_name, self.api.funnels.iter().map(|f| f.0).collect::<Vec<_>>()),
        }
    }

    #[tool(description = "Analyze cohort retention")]
    async fn analyze_cohort(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        serde_json::to_string_pretty(&self.api.generate_cohort()).unwrap()
    }

    #[tool(description = "Detect anomalies in a metric's recent data")]
    async fn detect_anomalies(&self, Parameters(input): Parameters<IdInput>) -> String {
        let anomalies = self.api.detect_anomalies_for(&input.id);
        serde_json::to_string_pretty(&anomalies).unwrap()
    }

    #[tool(description = "Forecast a metric N days forward with confidence intervals")]
    async fn forecast_metric(&self, Parameters(input): Parameters<ForecastInput>) -> String {
        let horizon = input.horizon_days.unwrap_or(14);
        let forecast = self.api.forecast_metric(&input.metric_id, horizon);
        serde_json::to_string_pretty(&forecast).unwrap()
    }

    #[tool(description = "Explain why a metric changed — dimension attribution analysis")]
    async fn explain_change(&self, Parameters(input): Parameters<ExplainInput>) -> String {
        let name = self.api.metrics.iter().find(|m| m.id == input.metric_id).map(|m| m.name.clone()).unwrap_or(input.metric_id.clone());
        serde_json::to_string_pretty(&serde_json::json!({
            "metric": name,
            "change": "-26% drop",
            "primary_driver": "Mobile traffic down 40% (device dimension)",
            "secondary_drivers": [
                "US region down 15%",
                "Free tier users down 30%"
            ],
            "likely_cause": "Mobile app crash on iOS 18.2 (deployed 3 days ago)",
            "confidence": 0.82
        })).unwrap()
    }

    #[tool(description = "Generate an AI-powered insight summary of key findings")]
    async fn generate_insight_summary(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let summary = InsightSummary {
            highlights: vec![
                "DAU grew 8% week-over-week, driven by organic signups".into(),
                "MRR reached $88K, up from $82K last month".into(),
                "Enterprise segment grew 12% (156 → 175 accounts)".into(),
            ],
            anomalies: vec![
                "DAU dropped 26% on May 22 — correlated with iOS app crash".into(),
                "Churn rate spiked to 5.1% for free tier users".into(),
            ],
            recommendations: vec![
                "Investigate iOS crash — 40% of mobile DAU affected".into(),
                "Free tier churn suggests onboarding friction — review funnel".into(),
                "Enterprise growth strong — consider dedicated success team".into(),
            ],
        };
        serde_json::to_string_pretty(&summary).unwrap()
    }

    // === Dashboard Building (5) ===

    #[tool(description = "List all dashboards")]
    async fn list_dashboards(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let dashes = self.api.dashboards.read().await;
        let summary: Vec<serde_json::Value> = dashes.iter().map(|d| {
            serde_json::json!({ "id": d.id, "name": d.name, "widgets": d.widgets.len(), "published": d.published })
        }).collect();
        serde_json::to_string_pretty(&summary).unwrap()
    }

    #[tool(description = "Get full dashboard definition with all widgets")]
    async fn get_dashboard(&self, Parameters(input): Parameters<IdInput>) -> String {
        let dashes = self.api.dashboards.read().await;
        match dashes.iter().find(|d| d.id == input.id) {
            Some(d) => serde_json::to_string_pretty(d).unwrap(),
            None => format!("Dashboard {} not found", input.id),
        }
    }

    #[tool(description = "Summarize a dashboard's key takeaways")]
    async fn summarize_dashboard(&self, Parameters(input): Parameters<IdInput>) -> String {
        let dashes = self.api.dashboards.read().await;
        match dashes.iter().find(|d| d.id == input.id) {
            Some(d) => {
                let widget_names: Vec<&str> = d.widgets.iter().map(|w| w.title.as_str()).collect();
                serde_json::to_string_pretty(&serde_json::json!({
                    "dashboard": d.name,
                    "widget_count": d.widgets.len(),
                    "widgets": widget_names,
                    "summary": format!("Dashboard '{}' tracks {} metrics. Key focus: growth and conversion.", d.name, d.widgets.len())
                })).unwrap()
            }
            None => format!("Dashboard {} not found", input.id),
        }
    }

    #[tool(description = "Create a new dashboard")]
    async fn create_dashboard(&self, Parameters(input): Parameters<CreateDashboardInput>) -> String {
        let id = format!("dash-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let dash = Dashboard {
            id: id.clone(),
            name: input.name.clone(),
            description: input.description.unwrap_or_default(),
            widgets: Vec::new(),
            published: false,
            created_by: "AI Agent".into(),
        };
        self.api.dashboards.write().await.push(dash);
        self.api.log_audit("create_dashboard", serde_json::json!({"name": input.name, "id": &id}), "allowed").await;
        format!("Created dashboard '{}' (id: {})", input.name, id)
    }

    #[tool(description = "Add a widget (line_chart, bar_chart, number, table, funnel) to a dashboard")]
    async fn add_widget(&self, Parameters(input): Parameters<AddWidgetInput>) -> String {
        let mut dashes = self.api.dashboards.write().await;
        match dashes.iter_mut().find(|d| d.id == input.dashboard_id) {
            Some(d) => {
                let wid = format!("w-{}", &uuid::Uuid::new_v4().to_string()[..8]);
                d.widgets.push(Widget {
                    id: wid.clone(),
                    widget_type: input.widget_type,
                    title: input.title.clone(),
                    metric_id: input.metric_id,
                    config: serde_json::json!({}),
                });
                format!("Added widget '{}' (id: {}) to dashboard {}", input.title, wid, input.dashboard_id)
            }
            None => format!("Dashboard {} not found", input.dashboard_id),
        }
    }

    #[tool(description = "Publish a dashboard — makes it visible to the team")]
    async fn publish_dashboard(&self, Parameters(input): Parameters<IdInput>) -> String {
        let mut dashes = self.api.dashboards.write().await;
        match dashes.iter_mut().find(|d| d.id == input.id) {
            Some(d) => {
                d.published = true;
                self.api.log_audit("publish_dashboard", serde_json::json!({"id": &input.id}), "allowed").await;
                format!("Dashboard '{}' published", d.name)
            }
            None => format!("Dashboard {} not found", input.id),
        }
    }

    // === Governance (5) ===

    #[tool(description = "Validate if an action is allowed by analytics policy")]
    async fn validate_analytics_policy(&self, Parameters(input): Parameters<PolicyCheckInput>) -> String {
        let ctx = input.context.unwrap_or(serde_json::json!({}));
        let (allowed, reason) = self.api.check_policy(&input.action, &ctx);
        self.api.log_audit("validate_analytics_policy", serde_json::json!({"action": input.action, "allowed": allowed}), if allowed { "allowed" } else { "denied" }).await;
        serde_json::to_string_pretty(&serde_json::json!({ "allowed": allowed, "reason": reason, "policy_action": input.action })).unwrap()
    }

    #[tool(description = "Assess risk level of exporting data from a dataset")]
    async fn check_export_risk(&self, Parameters(input): Parameters<ExportRiskInput>) -> String {
        let has_pii = input.includes_pii.unwrap_or_else(|| {
            self.api.datasets.iter().find(|d| d.id == input.dataset_id).map(|d| d.columns.iter().any(|c| c.is_pii)).unwrap_or(false)
        });
        let rows = input.row_count.unwrap_or(1000);
        let risk = if has_pii { "high" } else if rows > self.api.policy.row_limit { "medium" } else { "low" };
        let allowed = risk != "high" || self.api.policy.pii_column_access == "allowed";
        serde_json::to_string_pretty(&serde_json::json!({
            "dataset_id": input.dataset_id,
            "risk_level": risk,
            "contains_pii": has_pii,
            "row_count": rows,
            "row_limit": self.api.policy.row_limit,
            "export_allowed": allowed,
            "recommendation": if has_pii { "Strip PII columns before export" } else { "Safe to export" }
        })).unwrap()
    }

    #[tool(description = "Request access to restricted data (PII, financial, employee)")]
    async fn request_data_access(&self, Parameters(input): Parameters<DataAccessInput>) -> String {
        self.api.log_audit("request_data_access", serde_json::json!({"resource": &input.resource, "reason": &input.reason}), "pending").await;
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "pending",
            "resource": input.resource,
            "reason": input.reason,
            "message": "Access request submitted. Requires admin approval.",
            "request_id": uuid::Uuid::new_v4().to_string()
        })).unwrap()
    }

    #[tool(description = "Get audit trail of analytics queries and actions")]
    async fn get_query_audit_trail(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let log = self.api.audit_log.read().await;
        let recent: Vec<&AuditEntry> = log.iter().rev().take(20).collect();
        serde_json::to_string_pretty(&recent).unwrap()
    }

    // === Segments (2) ===

    #[tool(description = "List all user segments")]
    async fn get_segments(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        serde_json::to_string_pretty(&self.api.segments).unwrap()
    }

    #[tool(description = "Query a segment — get user count and definition")]
    async fn query_segment(&self, Parameters(input): Parameters<SegmentQueryInput>) -> String {
        match self.api.segments.iter().find(|s| s.id == input.segment_id) {
            Some(s) => serde_json::to_string_pretty(s).unwrap(),
            None => format!("Segment {} not found", input.segment_id),
        }
    }
}
