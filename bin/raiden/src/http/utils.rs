use std::sync::Arc;

use hyper::{
    Body,
    Request,
};
use parking_lot::RwLock;
use raiden::{
    api::Api,
    blockchain::{
        contracts::ContractsManager,
        proxies::ProxyManager,
    },
    state_manager::StateManager,
};
use routerify::ext::RequestExt;
use serde::de::DeserializeOwned;

use super::error::Error;

pub(crate) fn api(req: &Request<Body>) -> Arc<Api> {
    req.data::<Arc<Api>>().unwrap().clone()
}

pub(crate) fn state_manager(req: &Request<Body>) -> Arc<RwLock<StateManager>> {
    req.data::<Arc<RwLock<StateManager>>>().unwrap().clone()
}

pub(crate) fn contracts_manager(req: &Request<Body>) -> Arc<ContractsManager> {
    req.data::<Arc<ContractsManager>>().unwrap().clone()
}

pub(crate) fn proxy_manager(req: &Request<Body>) -> Arc<ProxyManager> {
    req.data::<Arc<ProxyManager>>().unwrap().clone()
}

pub(crate) async fn body_to_params<T: DeserializeOwned>(req: Request<Body>) -> Result<T, Error> {
    let body = hyper::body::to_bytes(req.into_body()).await.map_err(Error::Http)?;
    let params: T = serde_json::from_slice(&body).map_err(Error::Serialization)?;
    Ok(params)
}
