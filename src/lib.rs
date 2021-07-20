use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tonic::{Request, Response, Status};

pub mod proto;
pub use proto::{
    arquivo_client::ArquivoClient,
    arquivo_server::{Arquivo, ArquivoServer},
    InsertRequest, InsertResponse,
    SearchRequest, SearchResponse,
};

#[derive(Debug)]
pub struct ArquivoSvc {
    // store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    backend: Option<String>,
    runtime: Option<String>,
    index: Option<String>,
}

impl Default for ArquivoSvc {
    fn default() -> Self {
        Self::new()
    }
}

impl ArquivoSvc {
    pub fn new() -> Self {
        ArquivoSvc {
            // store: Arc::new(RwLock::new(HashMap::new())),
            backend: None,
            runtime: None,
            index: None,
        }
    }
}

#[tonic::async_trait]
impl Arquivo for ArquivoSvc {
    #[tracing::instrument]
    async fn insert(&self, request: Request<InsertRequest>) -> Result<Response<InsertResponse>, Status> {
        tracing::info!(
            "Got a Insert request from {:?}: {:?}",
            request.remote_addr(),
            request.get_ref()
        );

        let _r = request.get_ref();
        // self.storage.set(&r.key, &r.value).await;

        let reply = InsertResponse {};
        Ok(Response::new(reply))
    }

    async fn search(&self, request: Request<SearchRequest>) -> Result<Response<SearchResponse>, Status> {
        tracing::info!(
            "Got a Search request from {:?}: {:?}",
            request.remote_addr(),
            request.get_ref()
        );

        let _r = request.get_ref();
        // self.storage.set(&r.key, &r.value).await;

        let reply = SearchResponse {};
        Ok(Response::new(reply))
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
