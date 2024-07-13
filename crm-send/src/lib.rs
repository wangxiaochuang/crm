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
pub struct NotificationService {
    inner: Arc<NotificationServiceInner>,
}

#[allow(unused)]
pub struct NotificationServiceInner {
    config: AppConfig,
    sender: mpsc::Sender<Msg>,
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
            self.send(stream.map_err(|status| anyhow::anyhow!(status.to_string())))
                .await
                .map_err(|e| Status::internal(e.to_string())),
        )))
    }
}

impl NotificationService {
    pub fn new(config: AppConfig) -> Self {
        let sender = dummy_send();
        let inner = NotificationServiceInner { config, sender };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }
}

impl Deref for NotificationService {
    type Target = NotificationServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
