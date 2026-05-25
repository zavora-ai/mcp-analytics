use crate::domain::*;
use chrono::{NaiveDate, Utc};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Local analytics API backend. Simulates a real analytics platform.
/// Data lives in memory — the MCP server doesn't persist anything.
#[derive(Clone)]
pub struct AnalyticsApi {
    pub data_sources: Vec<DataSource>,
    pub datasets: Vec<Dataset>,
    pub metrics: Vec<MetricDefinition>,
    pub segments: Vec<Segment>,
    pub funnels: Vec<(&'static str, Vec<(&'static str, u64)>)>,
    pub dashboards: Arc<RwLock<Vec<Dashboard>>>,
    pub audit_log: Arc<RwLock<Vec<AuditEntry>>>,
    pub policy: Policy,
}

impl AnalyticsApi {
    pub fn seeded() -> Self {
        let data_sources = vec![
            DataSource { id: "ds-1".into(), name: "Product Database".into(), source_type: "product_db".into(), status: "connected".into() },
            DataSource { id: "ds-2".into(), name: "Data Warehouse".into(), source_type: "warehouse".into(), status: "connected".into() },
            DataSource { id: "ds-3".into(), name: "Payment Events".into(), source_type: "event_stream".into(), status: "connected".into() },
        ];
        let datasets = vec![
            Dataset { id: "tbl-users".into(), source_id: "ds-1".into(), name: "users".into(), row_count: 48500, columns: vec![
                Column { name: "user_id".into(), data_type: "uuid".into(), is_pii: false },
                Column { name: "email".into(), data_type: "string".into(), is_pii: true },
                Column { name: "plan".into(), data_type: "string".into(), is_pii: false },
                Column { name: "created_at".into(), data_type: "timestamp".into(), is_pii: false },
                Column { name: "country".into(), data_type: "string".into(), is_pii: false },
            ], freshness: "2 min ago".into() },
            Dataset { id: "tbl-events".into(), source_id: "ds-2".into(), name: "events".into(), row_count: 2_340_000, columns: vec![
                Column { name: "event_id".into(), data_type: "uuid".into(), is_pii: false },
                Column { name: "user_id".into(), data_type: "uuid".into(), is_pii: false },
                Column { name: "event_name".into(), data_type: "string".into(), is_pii: false },
                Column { name: "timestamp".into(), data_type: "timestamp".into(), is_pii: false },
                Column { name: "properties".into(), data_type: "json".into(), is_pii: false },
            ], freshness: "30 sec ago".into() },
            Dataset { id: "tbl-purchases".into(), source_id: "ds-3".into(), name: "purchases".into(), row_count: 12800, columns: vec![
                Column { name: "purchase_id".into(), data_type: "uuid".into(), is_pii: false },
                Column { name: "user_id".into(), data_type: "uuid".into(), is_pii: false },
                Column { name: "amount_cents".into(), data_type: "integer".into(), is_pii: false },
                Column { name: "plan".into(), data_type: "string".into(), is_pii: false },
                Column { name: "created_at".into(), data_type: "timestamp".into(), is_pii: false },
            ], freshness: "5 min ago".into() },
            Dataset { id: "tbl-subscriptions".into(), source_id: "ds-1".into(), name: "subscriptions".into(), row_count: 9200, columns: vec![
                Column { name: "sub_id".into(), data_type: "uuid".into(), is_pii: false },
                Column { name: "user_id".into(), data_type: "uuid".into(), is_pii: false },
                Column { name: "plan".into(), data_type: "string".into(), is_pii: false },
                Column { name: "mrr_cents".into(), data_type: "integer".into(), is_pii: false },
                Column { name: "status".into(), data_type: "string".into(), is_pii: false },
            ], freshness: "10 min ago".into() },
        ];
        let metrics = vec![
            MetricDefinition { id: "m-dau".into(), name: "DAU".into(), formula: "COUNT(DISTINCT user_id) WHERE event_date = today".into(), aggregation: "count_distinct".into(), dataset_id: "tbl-events".into(), dimensions: vec!["country".into(), "plan".into(), "device".into()], owner: "Growth Team".into(), certified: true },
            MetricDefinition { id: "m-wau".into(), name: "WAU".into(), formula: "COUNT(DISTINCT user_id) WHERE event_date >= today - 7".into(), aggregation: "count_distinct".into(), dataset_id: "tbl-events".into(), dimensions: vec!["country".into(), "plan".into()], owner: "Growth Team".into(), certified: true },
            MetricDefinition { id: "m-mau".into(), name: "MAU".into(), formula: "COUNT(DISTINCT user_id) WHERE event_date >= today - 30".into(), aggregation: "count_distinct".into(), dataset_id: "tbl-events".into(), dimensions: vec!["country".into(), "plan".into()], owner: "Growth Team".into(), certified: true },
            MetricDefinition { id: "m-mrr".into(), name: "MRR".into(), formula: "SUM(mrr_cents) / 100 WHERE status = 'active'".into(), aggregation: "sum".into(), dataset_id: "tbl-subscriptions".into(), dimensions: vec!["plan".into(), "country".into()], owner: "Finance".into(), certified: true },
            MetricDefinition { id: "m-signups".into(), name: "Signups".into(), formula: "COUNT(*) WHERE event_name = 'signup'".into(), aggregation: "count".into(), dataset_id: "tbl-events".into(), dimensions: vec!["country".into(), "source".into()], owner: "Growth Team".into(), certified: true },
            MetricDefinition { id: "m-churn".into(), name: "Churn Rate".into(), formula: "COUNT(churned) / COUNT(active_start) * 100".into(), aggregation: "avg".into(), dataset_id: "tbl-subscriptions".into(), dimensions: vec!["plan".into()], owner: "Finance".into(), certified: true },
            MetricDefinition { id: "m-arpu".into(), name: "ARPU".into(), formula: "SUM(amount_cents) / COUNT(DISTINCT user_id) / 100".into(), aggregation: "avg".into(), dataset_id: "tbl-purchases".into(), dimensions: vec!["plan".into(), "country".into()], owner: "Finance".into(), certified: true },
            MetricDefinition { id: "m-conversion".into(), name: "Conversion Rate".into(), formula: "COUNT(purchased) / COUNT(signed_up) * 100".into(), aggregation: "avg".into(), dataset_id: "tbl-events".into(), dimensions: vec!["source".into(), "plan".into()], owner: "Product".into(), certified: false },
        ];
        let segments = vec![
            Segment { id: "seg-1".into(), name: "Power Users".into(), definition: "events_last_7d >= 20".into(), count: 3200 },
            Segment { id: "seg-2".into(), name: "Churned".into(), definition: "last_active > 30 days ago AND was_paying".into(), count: 890 },
            Segment { id: "seg-3".into(), name: "New This Week".into(), definition: "created_at >= 7 days ago".into(), count: 420 },
            Segment { id: "seg-4".into(), name: "Enterprise".into(), definition: "plan = 'enterprise'".into(), count: 156 },
            Segment { id: "seg-5".into(), name: "Free Tier".into(), definition: "plan = 'free'".into(), count: 28400 },
        ];
        let funnels = vec![
            ("Signup to Purchase", vec![("Landing Page", 10000), ("Signup", 3200), ("Onboarding Complete", 2100), ("First Purchase", 840)]),
            ("Feature Adoption", vec![("Login", 5000), ("Feature Viewed", 3800), ("Feature Used", 2200), ("Repeated Use", 1100)]),
        ];
        let dashboards = vec![
            Dashboard {
                id: "dash-1".into(), name: "Growth Overview".into(), description: "Key growth metrics for the team".into(), published: true, created_by: "Growth Team".into(),
                widgets: vec![
                    Widget { id: "w-1".into(), widget_type: "number".into(), title: "DAU".into(), metric_id: Some("m-dau".into()), config: serde_json::json!({}) },
                    Widget { id: "w-2".into(), widget_type: "line_chart".into(), title: "DAU Trend (30d)".into(), metric_id: Some("m-dau".into()), config: serde_json::json!({"range": "30d", "granularity": "daily"}) },
                    Widget { id: "w-3".into(), widget_type: "number".into(), title: "Signups Today".into(), metric_id: Some("m-signups".into()), config: serde_json::json!({}) },
                    Widget { id: "w-4".into(), widget_type: "funnel".into(), title: "Signup Funnel".into(), metric_id: None, config: serde_json::json!({"funnel": "Signup to Purchase"}) },
                ],
            },
        ];
        Self {
            data_sources,
            datasets,
            metrics,
            segments,
            funnels,
            dashboards: Arc::new(RwLock::new(dashboards)),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            policy: Policy::default(),
        }
    }

    pub fn generate_time_series(&self, metric_id: &str, days: u32, granularity: &str) -> Vec<TimeSeriesPoint> {
        let mut rng = rand::thread_rng();
        let base = match metric_id {
            "m-dau" => 4200.0,
            "m-wau" => 12000.0,
            "m-mau" => 32000.0,
            "m-mrr" => 85000.0,
            "m-signups" => 150.0,
            "m-churn" => 3.2,
            "m-arpu" => 42.0,
            "m-conversion" => 8.5,
            _ => 1000.0,
        };
        let step = match granularity {
            "weekly" => 7,
            "monthly" => 30,
            _ => 1,
        };
        let today = Utc::now().date_naive();
        let mut points = Vec::new();
        let mut i = days as i64;
        while i > 0 {
            let date = today - chrono::Duration::days(i);
            let trend = (days as f64 - i as f64) / days as f64 * 0.1; // slight uptrend
            let noise = rng.gen_range(-0.05..0.05);
            let value = base * (1.0 + trend + noise);
            points.push(TimeSeriesPoint { date: date.to_string(), value: (value * 100.0).round() / 100.0 });
            i -= step as i64;
        }
        points
    }

    pub fn generate_breakdown(&self, metric_id: &str, dimension: &str) -> Vec<SegmentData> {
        match dimension {
            "plan" => vec![
                SegmentData { label: "Free".into(), value: 58.0, percentage: 58.0 },
                SegmentData { label: "Pro".into(), value: 28.0, percentage: 28.0 },
                SegmentData { label: "Enterprise".into(), value: 14.0, percentage: 14.0 },
            ],
            "country" => vec![
                SegmentData { label: "US".into(), value: 42.0, percentage: 42.0 },
                SegmentData { label: "UK".into(), value: 18.0, percentage: 18.0 },
                SegmentData { label: "DE".into(), value: 12.0, percentage: 12.0 },
                SegmentData { label: "Other".into(), value: 28.0, percentage: 28.0 },
            ],
            "device" => vec![
                SegmentData { label: "Desktop".into(), value: 62.0, percentage: 62.0 },
                SegmentData { label: "Mobile".into(), value: 30.0, percentage: 30.0 },
                SegmentData { label: "Tablet".into(), value: 8.0, percentage: 8.0 },
            ],
            _ => vec![
                SegmentData { label: "Segment A".into(), value: 60.0, percentage: 60.0 },
                SegmentData { label: "Segment B".into(), value: 40.0, percentage: 40.0 },
            ],
        }
    }

    pub fn generate_cohort(&self) -> CohortResult {
        CohortResult {
            cohort_size: "weekly".into(),
            cohorts: vec![
                CohortRow { cohort: "Week -4".into(), size: 380, retention: vec![100.0, 62.0, 48.0, 41.0] },
                CohortRow { cohort: "Week -3".into(), size: 410, retention: vec![100.0, 65.0, 50.0] },
                CohortRow { cohort: "Week -2".into(), size: 395, retention: vec![100.0, 60.0] },
                CohortRow { cohort: "Week -1".into(), size: 430, retention: vec![100.0] },
            ],
        }
    }

    pub fn detect_anomalies_for(&self, metric_id: &str) -> Vec<Anomaly> {
        let name = self.metrics.iter().find(|m| m.id == metric_id).map(|m| m.name.clone()).unwrap_or(metric_id.into());
        vec![
            Anomaly { date: (Utc::now().date_naive() - chrono::Duration::days(3)).to_string(), metric_name: name.clone(), expected: 4200.0, actual: 3100.0, severity: "high".into() },
            Anomaly { date: (Utc::now().date_naive() - chrono::Duration::days(7)).to_string(), metric_name: name, expected: 4200.0, actual: 4800.0, severity: "low".into() },
        ]
    }

    pub fn forecast_metric(&self, metric_id: &str, horizon_days: u32) -> Forecast {
        let mut rng = rand::thread_rng();
        let base = match metric_id {
            "m-dau" => 4400.0,
            "m-mrr" => 88000.0,
            "m-signups" => 160.0,
            _ => 1000.0,
        };
        let name = self.metrics.iter().find(|m| m.id == metric_id).map(|m| m.name.clone()).unwrap_or(metric_id.into());
        let today = Utc::now().date_naive();
        let mut predictions = Vec::new();
        let mut lower = Vec::new();
        let mut upper = Vec::new();
        for d in 1..=horizon_days {
            let trend = d as f64 / horizon_days as f64 * 0.05;
            let val = base * (1.0 + trend + rng.gen_range(-0.02..0.02));
            predictions.push(TimeSeriesPoint { date: (today + chrono::Duration::days(d as i64)).to_string(), value: (val * 100.0).round() / 100.0 });
            lower.push((val * 0.9 * 100.0).round() / 100.0);
            upper.push((val * 1.1 * 100.0).round() / 100.0);
        }
        Forecast { metric_name: name, horizon_days, predictions, confidence_lower: lower, confidence_upper: upper }
    }

    pub async fn log_audit(&self, tool: &str, params: serde_json::Value, result: &str) {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            tool: tool.into(),
            params,
            result: result.into(),
        };
        self.audit_log.write().await.push(entry);
    }

    pub fn check_policy(&self, action: &str, context: &serde_json::Value) -> (bool, String) {
        match action {
            "raw_sql" => {
                if !self.policy.raw_sql_allowed { return (false, "Raw SQL queries are not allowed by policy".into()); }
            }
            "pii_access" => {
                if self.policy.pii_column_access == "denied" { return (false, "PII column access is denied by policy".into()); }
            }
            "customer_export" => {
                if self.policy.customer_level_export == "denied" { return (false, "Customer-level export is denied".into()); }
                if self.policy.customer_level_export == "requires_approval" { return (false, "Customer-level export requires approval".into()); }
            }
            "employee_export" => {
                if self.policy.employee_level_export == "denied" { return (false, "Employee-level export is denied".into()); }
            }
            "financial_access" => {
                if self.policy.financial_metric_access == "denied" { return (false, "Financial metric access is denied".into()); }
                if self.policy.financial_metric_access == "approved_only" {
                    let approved = context.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);
                    if !approved { return (false, "Financial metric access requires prior approval".into()); }
                }
            }
            "external_share" => {
                if !self.policy.external_sharing_allowed { return (false, "External sharing is not allowed".into()); }
            }
            _ => {}
        }
        (true, "Allowed".into())
    }
}
