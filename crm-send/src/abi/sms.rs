use anyhow::Result;

use crate::{
    pb::{send_request::Msg, SendRequest, SendResponse, SmsMessage},
    AppState,
};

use super::{to_ts, Sender};

impl Sender for SmsMessage {
    async fn send(self, state: AppState) -> Result<SendResponse> {
        let message_id = self.message_id.clone();
        state.sender.send(Msg::Sms(self)).await?;
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<SmsMessage> for Msg {
    fn from(sms: SmsMessage) -> Self {
        Msg::Sms(sms)
    }
}

impl From<SmsMessage> for SendRequest {
    fn from(sms: SmsMessage) -> Self {
        let msg: Msg = sms.into();
        SendRequest { msg: Some(msg) }
    }
}

#[cfg(test)]
impl SmsMessage {
    pub fn fake() -> Self {
        use fake::faker::phone_number::en::PhoneNumber;
        use fake::Fake;
        use uuid::Uuid;
        SmsMessage {
            message_id: Uuid::new_v4().to_string(),
            sender: PhoneNumber().fake(),
            recipients: vec![PhoneNumber().fake()],
            body: "Hello, world!".to_string(),
        }
    }
}
