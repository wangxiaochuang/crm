use std::{ops::Deref, pin::Pin, sync::Arc};

mod abi;
mod config;
pub mod pb;

pub use config::AppConfig;
use futures::Stream;
use pb::{
    metadata_server::{Metadata, MetadataServer},
    Content, MaterializeRequest,
};
use tonic::{async_trait, Request, Response, Result, Streaming};

pub struct MetadataService {
    inner: Arc<MetadataServiceInner>,
}

pub struct MetadataServiceInner {
    #[allow(dead_code)]
    config: AppConfig,
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
        self.materialize(query).await
    }
}

impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(MetadataServiceInner { config }),
        }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}

impl Deref for MetadataService {
    type Target = MetadataServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
