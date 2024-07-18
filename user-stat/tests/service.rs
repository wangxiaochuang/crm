use std::net::SocketAddr;

use anyhow::Result;
use futures::{future, StreamExt as _};
use sqlx_db_tester::TestPg;
use tokio::net::TcpListener;
use tonic::transport::Server;
use user_stat::{
    pb::{user_stats_client::UserStatsClient, QueryRequestBuilder, RawQueryRequestBuilder},
    test_util::{id, tq},
    AppState, UserStatsService,
};

#[tokio::test]
async fn raw_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server().await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;
    let req = RawQueryRequestBuilder::default()
        .query("SELECT * FROM user_stats WHERE created_at > '2024-01-01' LIMIT 5")
        .build()?;

    let stream = client.raw_query(req).await?.into_inner();
    let ret = stream
        .filter_map(|user| future::ready(user.ok()))
        .collect::<Vec<_>>()
        .await;

    assert_eq!(ret.len(), 5);

    Ok(())
}

#[tokio::test]
async fn query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server().await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;
    let req = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), tq(Some(1200), None)))
        .timestamp(("last_visited_at".to_string(), tq(Some(3000), None)))
        .id(("viewed_but_not_started".to_string(), id(&[232939])))
        .build()
        .unwrap();

    let stream = client.query(req).await?.into_inner();
    let ret = stream
        .filter_map(|user| future::ready(user.ok()))
        .collect::<Vec<_>>()
        .await;

    assert_eq!(ret.len(), 1);

    Ok(())
}

async fn start_server() -> Result<(TestPg, SocketAddr)> {
    let (state, tdb) = AppState::new_for_test().await?;
    let addr = format!("0.0.0.0:{}", state.config.server.port);
    let svc = UserStatsService::new(state);
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

    Ok((tdb, addr))
}
