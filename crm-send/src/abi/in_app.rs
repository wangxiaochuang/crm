use anyhow::Result;

use crate::{
    pb::{send_request::Msg, InAppMessage, SendRequest, SendResponse},
    AppState,
};

use super::{to_ts, Sender};

impl Sender for InAppMessage {
    async fn send(self, state: AppState) -> Result<SendResponse> {
        let message_id = self.message_id.clone();
        state.sender.send(Msg::InApp(self)).await?;
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<InAppMessage> for Msg {
    fn from(in_app: InAppMessage) -> Self {
        Msg::InApp(in_app)
    }
}

impl From<InAppMessage> for SendRequest {
    fn from(in_app: InAppMessage) -> Self {
        let msg: Msg = in_app.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(test)]
impl InAppMessage {
    pub fn fake() -> Self {
        use uuid::Uuid;
        InAppMessage {
            message_id: Uuid::new_v4().to_string(),
            device_id: Uuid::new_v4().to_string(),
            title: "Hello".to_string(),
            body: "Hello, world!".to_string(),
        }
    }
}
