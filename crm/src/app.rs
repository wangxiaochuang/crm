use crate::config::AppConfig;
use anyhow::Result;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;
use std::{ops::Deref, str::FromStr as _, sync::Arc};
use tonic::transport::{Channel, Uri};
use user_stat::pb::user_stats_client::UserStatsClient;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub struct AppStateInner {
    pub config: AppConfig,
    pub user_stats: UserStatsClient<Channel>,
    pub notification: NotificationClient<Channel>,
    pub metadata: MetadataClient<Channel>,
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        let user_stats =
            UserStatsClient::connect(Uri::from_str(&config.server.user_stats)?).await?;
        let notification =
            NotificationClient::connect(Uri::from_str(&config.server.notification)?).await?;
        let metadata = MetadataClient::connect(Uri::from_str(&config.server.metadata)?).await?;
        let inner = AppStateInner {
            config,
            user_stats,
            notification,
            metadata,
        };
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
