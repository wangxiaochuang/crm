use anyhow::Result;
use crm::{app::AppState, config::AppConfig, CrmService};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::try_load().expect("Failed to load config");
    let addr = format!("[::1]:{}", config.server.port).parse().unwrap();
    let state = AppState::try_new(config).await?;
    info!("UserService listening on {}", addr);

    let svc = CrmService::new(state).into_server();
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
