use std::{ops::Deref, pin::Pin, sync::Arc};

mod abi;
mod config;
pub mod pb;

pub use config::AppConfig;
use futures::Stream;
use pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest, User,
};
use sqlx::PgPool;
use tonic::{async_trait, Request, Response, Result, Status};

pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

pub struct UserStatsServiceInner {
    #[allow(dead_code)]
    config: AppConfig,
    pool: PgPool,
}

type ServiceResult<T> = Result<Response<T>>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User>> + Send>>;

#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    type RawQueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        // method 1: self.query(query).await
        // method 2: self.query(query)
        //     .await
        //     .map(|ret| Response::new(Box::pin(futures::stream::iter(ret.into_iter().map(Ok)))))
        //     .map_err(|e| Status::internal(e.to_string()))
        // method 3:
        match self.query(query).await {
            Ok(ret) => Ok(Response::new(Box::pin(futures::stream::iter(
                ret.into_iter().map(Ok),
            )))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        // self.raw_query(query).await
        match self.raw_query(query).await {
            Ok(ret) => Ok(Response::new(Box::pin(futures::stream::iter(
                ret.into_iter().map(Ok),
            )))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

impl UserStatsService {
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed to connect to db");
        let inner = UserStatsServiceInner { config, pool };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
