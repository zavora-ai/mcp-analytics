mod api;
mod domain;
mod server;

use api::AnalyticsApi;
use rmcp::{ServiceExt, transport::stdio};
use server::AnalyticsServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let service = AnalyticsServer { api: AnalyticsApi::seeded() }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
