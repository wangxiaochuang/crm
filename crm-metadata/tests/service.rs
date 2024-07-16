use std::net::SocketAddr;

use anyhow::Result;
use crm_metadata::{
    pb::{metadata_client::MetadataClient, MaterializeRequest},
    AppState, MetadataService,
};
use futures::{future, StreamExt as _};
use tokio::net::TcpListener;
use tonic::transport::Server;

#[tokio::test]
async fn test_metadata() -> Result<()> {
    let addr = start_server().await?;
    let mut client = MetadataClient::connect(format!("http://{addr}")).await?;
    let stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
        MaterializeRequest { id: 3 },
    ]);

    let stream = client.materialize(stream).await?.into_inner();
    let ret = stream
        .filter_map(|user| future::ready(user.ok()))
        .collect::<Vec<_>>()
        .await;

    assert_eq!(ret.len(), 3);

    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let state = AppState::new_for_test().await?;
    let addr = format!("0.0.0.0:{}", state.config.server.port);
    let svc = MetadataService::new(state);
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
