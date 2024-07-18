use core::fmt;
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use futures::Stream;
use itertools::Itertools;
use prost_types::Timestamp;

use crate::{
    pb::{QueryRequest, QueryRequestBuilder, RawQueryRequest, TimeQuery, User},
    AppState,
};

impl QueryRequest {
    pub fn new_with_dt(name: &str, lower: DateTime<Utc>, upper: DateTime<Utc>) -> Self {
        // let ts = Timestamp { seconds: lower.timestamp(), nanos: 0, };
        let tq = TimeQuery {
            lower: Some(SystemTime::from(lower).into()),
            upper: Some(SystemTime::from(upper).into()),
        };
        QueryRequestBuilder::default()
            .timestamp((name.to_string(), tq))
            .build()
            .expect("Failed to build query request")
    }
}

impl fmt::Display for QueryRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let conditions = self
            .timestamps
            .iter()
            .map(|(k, v)| timestamp_query(k, v.lower.as_ref(), v.upper.as_ref()))
            .chain(self.ids.iter().map(|(k, v)| ids_query(k, &v.ids)))
            .join(" AND ");

        write!(f, "SELECT email, name FROM user_stats WHERE {}", conditions)
    }
}

impl AppState {
    pub async fn query(&self, query: QueryRequest) -> Result<impl Stream<Item = Result<User>>> {
        self.raw_query(RawQueryRequest {
            query: query.to_string(),
        })
        .await
    }

    pub async fn raw_query(
        &self,
        req: RawQueryRequest,
    ) -> Result<impl Stream<Item = Result<User>>> {
        // TODO: query must only return email and name, so we should use sqlparser to parse the query
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            return Err(anyhow!(format!(
                "Failed to fetch data with query: {}",
                req.query
            )));
        };

        Ok(futures::stream::iter(ret.into_iter().map(Ok)))
    }
}

fn ids_query(name: &str, ids: &[u32]) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

fn timestamp_query(name: &str, lower: Option<&Timestamp>, upper: Option<&Timestamp>) -> String {
    match (lower, upper) {
        (None, None) => "TRUE".to_string(),
        (None, Some(upper)) => format!("{} <= '{}'", name, ts_to_utc(upper).to_rfc3339()),
        (Some(lower), None) => format!("{} >= '{}'", name, ts_to_utc(lower).to_rfc3339()),
        (Some(lower), Some(upper)) => format!(
            "{} BETWEEN '{}' AND '{}'",
            name,
            ts_to_utc(lower).to_rfc3339(),
            ts_to_utc(upper).to_rfc3339()
        ),
    }
}

fn ts_to_utc(ts: &Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}

#[cfg(test)]
mod tests {
    use futures::{future, StreamExt as _};

    use crate::{
        pb::QueryRequestBuilder,
        test_util::{id, tq},
    };

    use super::*;

    #[test]
    fn query_request_to_string_should_work() {
        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let sql = query.to_string();
        assert_eq!(
            sql,
            "SELECT email, name FROM user_stats WHERE created_at BETWEEN '2024-01-01T00:00:00+00:00' AND '2024-01-02T00:00:00+00:00'"
        );
    }

    #[tokio::test]
    async fn raw_query_should_work() -> Result<()> {
        let (state, _tdb) = AppState::new_for_test().await?;
        let stream = state
            .raw_query(RawQueryRequest {
                query: "select * from user_stats where created_at > '2024-01-01' limit 5"
                    .to_string(),
            })
            .await?;
        // while let Some(Ok(user)) = stream.next().await {
        //     println!("{:?}", user);
        // }
        let users = stream
            // .filter_map(|user| async move { user.ok() })
            .filter_map(|user| future::ready(user.ok()))
            .collect::<Vec<_>>()
            .await;
        assert_eq!(users.len(), 5);
        Ok(())
    }

    #[tokio::test]
    async fn query_should_work() -> Result<()> {
        let (state, _tdb) = AppState::new_for_test().await?;
        let query = QueryRequestBuilder::default()
            .timestamp(("created_at".to_string(), tq(Some(1200), None)))
            .timestamp(("last_visited_at".to_string(), tq(Some(3000), None)))
            .id(("viewed_but_not_started".to_string(), id(&[250788])))
            .build()
            .unwrap();
        let stream = state.query(query).await?;
        let users = stream
            // .filter_map(|user| async move { user.ok() })
            .filter_map(|user| future::ready(user.ok()))
            .collect::<Vec<_>>()
            .await;
        assert_eq!(users.len(), 1);
        Ok(())
    }
}
