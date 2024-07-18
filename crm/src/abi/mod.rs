use std::sync::Arc;

use crate::{
    app::AppState,
    pb::{WelcomeRequest, WelcomeResponse},
};
use anyhow::Result;
use chrono::{Duration, Utc};
use crm_metadata::pb::MaterializeRequest;
use crm_send::pb::SendRequest;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::warn;
use user_stat::pb::QueryRequest;

impl AppState {
    pub async fn welcome(&self, req: WelcomeRequest) -> Result<WelcomeResponse> {
        let request_id = req.id;
        let d1 = Utc::now() - Duration::days(req.interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.content_ids))
            .await?
            .into_inner()
            .filter_map(|v| async move { v.ok() })
            .collect::<Vec<_>>()
            .await;
        let contents = Arc::new(contents);

        let sender = self.config.server.sender_email.clone();

        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(async move {
            let sender = sender.clone();
            while let Some(Ok(user)) = res_user_stats.next().await {
                let req =
                    SendRequest::new("Welcome".to_string(), &sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        });
        let reqs = ReceiverStream::new(rx);
        // let reqs = res_user_stats.filter_map(move |v| async {
        //     let sender = sender.clone();
        //     v.ok().map(|user| {
        //         SendRequest::new("Welcome".to_string(), &sender, &[user.email], &contents)
        //     })
        // });
        self.notification.clone().send(reqs).await?;
        Ok(WelcomeResponse { id: request_id })
    }
}
