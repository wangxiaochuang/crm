use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use futures::Stream;
use itertools::Itertools;
use prost_types::Timestamp;

use crate::{
    pb::{QueryRequest, RawQueryRequest, User},
    UserStatsService,
};

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> Result<impl Stream<Item = Result<User>>> {
        // generate sql based on query
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();

        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower, v.upper))
            .join(" AND ");

        sql.push_str(&time_conditions);

        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .join(" AND ");

        sql.push_str(" AND ");
        sql.push_str(&id_conditions);

        println!("Generated SQL: {}", sql);

        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(
        &self,
        req: RawQueryRequest,
    ) -> Result<impl Stream<Item = Result<User>>> {
        // TODO: query must only return email and name, so we should use sqlparser to parse the query
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.inner.pool)
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

fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

fn timestamp_query(name: &str, lower: Option<Timestamp>, upper: Option<Timestamp>) -> String {
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

fn ts_to_utc(ts: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}
