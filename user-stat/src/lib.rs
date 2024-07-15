use std::{ops::Deref, pin::Pin, sync::Arc};

mod abi;
mod config;
pub mod pb;

pub use config::AppConfig;
use futures::{Stream, TryStreamExt};
use pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest, User,
};
use sqlx::PgPool;
use tonic::{async_trait, Request, Response, Result, Status};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pool: PgPool,
}

pub struct UserStatsService {
    state: AppState,
}

type ServiceResult<T> = Result<Response<T>>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User>> + Send>>;

#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    type RawQueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();

        match self.state.query(query).await {
            Ok(stream) => Ok(Response::new(Box::pin(
                stream.map_err(|e| Status::internal(e.to_string())),
            ))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();

        match self.state.raw_query(query).await {
            Ok(stream) => Ok(Response::new(Box::pin(
                stream.map_err(|e| Status::internal(e.to_string())),
            ))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

impl UserStatsService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed to connect to db");
        let inner = AppStateInner { pool };
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
    use std::path::Path;
    use std::sync::Arc;

    use crate::pb::IdQuery;
    use crate::pb::TimeQuery;
    use crate::AppState;
    use crate::AppStateInner;
    use anyhow::Result;
    use chrono::Utc;
    use prost_types::Timestamp;
    use sqlx::Executor;
    use sqlx::PgPool;
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test() -> Result<(Self, sqlx_db_tester::TestPg)> {
            let (tdb, pool) = get_test_pool().await;
            let svc = Self {
                inner: Arc::new(AppStateInner { pool }),
            };
            Ok((svc, tdb))
        }
    }

    pub async fn get_test_pool() -> (TestPg, PgPool) {
        let url = "postgres://postgres:postgres@localhost:5432".to_owned();

        let tdb = TestPg::new(url, Path::new("./migrations"));
        let pool = tdb.get_pool().await;

        let sqls = include_str!("../fixtures/test.sql")
            .split(';')
            .filter(|sql| !sql.trim().is_empty());

        let mut ts = pool.begin().await.expect("begin transaction failed");
        for sql in sqls {
            ts.execute(sql).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }

    pub fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }

    pub fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }

    pub fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}
