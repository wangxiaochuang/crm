use anyhow::Result;
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};
use user_stat::{AppConfig, UserStatsService};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::try_load().expect("Failed to load config");
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    info!("UserService listening on {}", addr);

    let svc = UserStatsService::new(config).await.into_server();
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
