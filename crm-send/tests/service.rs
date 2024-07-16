use std::net::SocketAddr;

use anyhow::Result;
use crm_send::{
    pb::{
        notification_client::NotificationClient, EmailMessage, InAppMessage, SendRequest,
        SmsMessage,
    },
    AppState, NotificationService,
};
use futures::{future, StreamExt};
use tokio::net::TcpListener;
use tonic::transport::Server;

#[tokio::test]
async fn test_metadata() -> Result<()> {
    let addr = start_server().await?;
    let mut client = NotificationClient::connect(format!("http://{addr}")).await?;
    let stream = tokio_stream::iter(vec![
        SendRequest {
            msg: Some(EmailMessage::fake().into()),
        },
        SendRequest {
            msg: Some(SmsMessage::fake().into()),
        },
        SendRequest {
            msg: Some(InAppMessage::fake().into()),
        },
    ]);

    let stream = client.send(stream).await?.into_inner();
    let ret = stream
        .filter_map(|user| future::ready(user.ok()))
        .collect::<Vec<_>>()
        .await;

    assert_eq!(ret.len(), 3);
    println!("{:?}", ret);

    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let state = AppState::new_for_test().await?;
    let addr = format!("0.0.0.0:{}", state.config.server.port);
    let svc = NotificationService::new(state);
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;
    let listener_stream = tokio_stream::wrappers::TcpListenerStream::new(listener);
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc.into_server())
            .serve_with_incoming(listener_stream)
            .await
            .unwrap();
    });

    Ok(addr)
}
