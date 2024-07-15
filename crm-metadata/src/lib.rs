use std::{ops::Deref, pin::Pin, sync::Arc};

mod abi;
mod config;
pub mod pb;

use anyhow::anyhow;
pub use config::AppConfig;
use futures::{Stream, TryStreamExt};
use pb::{
    metadata_server::{Metadata, MetadataServer},
    Content, MaterializeRequest,
};
use tonic::{async_trait, Request, Response, Result, Status, Streaming};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    #[allow(dead_code)]
    config: AppConfig,
}

pub struct MetadataService {
    state: AppState,
}

type ServiceResult<T> = Result<Response<T>>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content>> + Send>>;
#[async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;
    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> ServiceResult<Self::MaterializeStream> {
        let query = request.into_inner();
        let stream = self.state.materialize(query.map_err(|e| anyhow!(e))).await;
        Ok(Response::new(Box::pin(
            stream.map_err(|e| Status::internal(e.to_string())),
        )))
    }
}

impl MetadataService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
