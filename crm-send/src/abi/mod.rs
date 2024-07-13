mod email;
mod in_app;
mod sms;

use anyhow::Result as MyResult;
use anyhow::Result;
use chrono::Utc;
use futures::{Stream, StreamExt};
use prost_types::Timestamp;
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};

use crate::{
    pb::{send_request::Msg, SendRequest, SendResponse},
    NotificationService,
};

pub trait Sender {
    async fn send(self, svc: NotificationService) -> MyResult<SendResponse>;
}

const CHANNEL_SIZE: usize = 1024;

impl NotificationService {
    pub async fn send(
        &self,
        mut stream: impl Stream<Item = Result<SendRequest>> + Send + 'static + Unpin,
    ) -> impl Stream<Item = Result<SendResponse>> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        let notif = self.clone();
        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                let notif_clone = notif.clone();
                let res = match req.msg {
                    Some(Msg::Email(email)) => email.send(notif_clone).await,
                    Some(Msg::Sms(sms)) => sms.send(notif_clone).await,
                    Some(Msg::InApp(in_app)) => in_app.send(notif_clone).await,
                    None => {
                        warn!("Invalid request");
                        Err(anyhow::anyhow!("Invalid request"))
                    }
                };
                tx.send(res).await.unwrap();
            }
        });

        ReceiverStream::new(rx)
    }
}

pub fn dummy_send() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(CHANNEL_SIZE * 100);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Sending message: {:?}", msg);
            sleep(Duration::from_millis(300)).await;
        }
    });
    tx
}

fn to_ts() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        pb::{EmailMessage, InAppMessage, SmsMessage},
        AppConfig,
    };
    use anyhow::Result;

    #[tokio::test]
    async fn send_should_work() -> Result<()> {
        let config = AppConfig::try_load()?;
        let service = NotificationService::new(config);
        let stream = tokio_stream::iter(vec![
            Ok(EmailMessage::fake().into()),
            Ok(SmsMessage::fake().into()),
            Ok(InAppMessage::fake().into()),
        ]);
        let response = service.send(stream).await;
        let ret = response.collect::<Vec<_>>().await;
        assert_eq!(ret.len(), 3);
        Ok(())
    }
}
