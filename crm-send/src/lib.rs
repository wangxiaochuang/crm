pub mod pb;

mod abi;
mod config;

use std::{ops::Deref, pin::Pin, sync::Arc};

use abi::dummy_send;
pub use config::AppConfig;
use futures::{Stream, TryStreamExt};
use pb::{
    notification_server::{Notification, NotificationServer},
    send_request::Msg,
    SendRequest, SendResponse,
};
use tokio::sync::mpsc;
use tonic::{async_trait, Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub struct AppStateInner {
    pub config: AppConfig,
    sender: mpsc::Sender<Msg>,
}

pub struct NotificationService {
    state: AppState,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;

#[async_trait]
impl Notification for NotificationService {
    type SendStream = ResponseStream;

    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> ServiceResult<Self::SendStream> {
        let stream = request.into_inner();
        Ok(Response::new(Box::pin(
            self.state
                .send(stream.map_err(|status| anyhow::anyhow!(status.to_string())))
                .await
                .map_err(|e| Status::internal(e.to_string())),
        )))
    }
}

impl NotificationService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let sender = dummy_send();
        let inner = AppStateInner { config, sender };
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(feature = "test_utils")]
pub mod test_util {
    use crate::{AppConfig, AppState};
    use anyhow::Result;

    const TEST_APP_YAML: &str = r#"
server:
  port: 0
"#;
    impl AppConfig {
        pub fn try_load_for_test() -> Result<Self> {
            let config_reader =
                std::io::BufReader::new(std::io::Cursor::new(TEST_APP_YAML.as_bytes()));
            Self::load_from_reader(config_reader)
        }
    }
    impl AppState {
        pub async fn new_for_test() -> Result<Self> {
            let config = AppConfig::try_load_for_test()?;
            Ok(Self::new(config))
        }
    }
}
