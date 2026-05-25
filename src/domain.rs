use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub source_type: String, // "warehouse", "product_db", "event_stream"
    pub status: String,      // "connected", "stale", "error"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub row_count: u64,
    pub columns: Vec<Column>,
    pub freshness: String, // "2 min ago", "1 hour ago"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_pii: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub id: String,
    pub name: String,
    pub formula: String,
    pub aggregation: String, // "count", "sum", "avg", "count_distinct"
    pub dataset_id: String,
    pub dimensions: Vec<String>,
    pub owner: String,
    pub certified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric_name: String,
    pub granularity: String,
    pub start_date: String,
    pub end_date: String,
    pub data: Vec<TimeSeriesPoint>,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakdownResult {
    pub metric_name: String,
    pub dimension: String,
    pub segments: Vec<SegmentData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentData {
    pub label: String,
    pub value: f64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelResult {
    pub name: String,
    pub steps: Vec<FunnelStep>,
    pub overall_conversion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    pub name: String,
    pub count: u64,
    pub conversion_from_prev: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortResult {
    pub cohort_size: String, // "weekly", "monthly"
    pub cohorts: Vec<CohortRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortRow {
    pub cohort: String,
    pub size: u64,
    pub retention: Vec<f64>, // percentages per period
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub widgets: Vec<Widget>,
    pub published: bool,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub widget_type: String, // "line_chart", "bar_chart", "number", "table", "funnel"
    pub title: String,
    pub metric_id: Option<String>,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: String,
    pub event_name: String,
    pub user_id: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub date: String,
    pub metric_name: String,
    pub expected: f64,
    pub actual: f64,
    pub severity: String, // "low", "medium", "high"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub metric_name: String,
    pub horizon_days: u32,
    pub predictions: Vec<TimeSeriesPoint>,
    pub confidence_lower: Vec<f64>,
    pub confidence_upper: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub tool: String,
    pub params: serde_json::Value,
    pub result: String, // "allowed", "denied", "rate_limited"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub raw_sql_allowed: bool,
    pub pii_column_access: String,       // "denied", "approved_only", "allowed"
    pub row_limit: u64,
    pub time_range_limit_days: u32,
    pub customer_level_export: String,   // "denied", "requires_approval", "allowed"
    pub employee_level_export: String,
    pub financial_metric_access: String,
    pub external_sharing_allowed: bool,
    pub metric_certification_required: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            raw_sql_allowed: false,
            pii_column_access: "denied".into(),
            row_limit: 10000,
            time_range_limit_days: 365,
            customer_level_export: "requires_approval".into(),
            employee_level_export: "denied".into(),
            financial_metric_access: "approved_only".into(),
            external_sharing_allowed: false,
            metric_certification_required: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub name: String,
    pub definition: String, // human-readable filter
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightSummary {
    pub highlights: Vec<String>,
    pub anomalies: Vec<String>,
    pub recommendations: Vec<String>,
}
