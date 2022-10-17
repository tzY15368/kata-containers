// Copyright (c) 2019-2022 Alibaba Cloud
// Copyright (c) 2019-2022 Ant Group
//
// SPDX-License-Identifier: Apache-2.0
//

// This defines the handlers corresponding to the url when a request is sent to destined url,
// the handler function should be invoked, and the corresponding data will be in the response

use common::Sandbox;
use hyper::{Body, Method, Request, Response, Result, StatusCode};
use std::sync::Arc;

use super::direct_volume::{
    ResizeRequest, DIRECT_VOLUME_PATH_KEY, DIRECT_VOLUME_RESIZE_URL, DIRECT_VOLUME_STAT_URL,
};
use super::server::AGENT_URL;
use url::Url;

// main router for response, this works as a multiplexer on
// http arrival which invokes the corresponding handler function
pub(crate) async fn handler_mux(
    sandbox: Arc<dyn Sandbox>,
    req: Request<Body>,
) -> Result<Response<Body>> {
    info!(
        sl!(),
        "mgmt-svr(mux): recv req, method: {}, uri: {}",
        req.method(),
        req.uri().path()
    );
    match (req.method(), req.uri().path()) {
        (&Method::GET, AGENT_URL) => agent_url_handler(sandbox, req).await,
        (&Method::GET, DIRECT_VOLUME_STAT_URL) => direct_volume_stats_handler(sandbox, req).await,
        (&Method::POST, DIRECT_VOLUME_RESIZE_URL) => {
            direct_volume_resize_handler(sandbox, req).await
        }
        _ => Ok(not_found(req).await),
    }
}

// url not found
async fn not_found(_req: Request<Body>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("URL NOT FOUND"))
        .unwrap()
}

// returns the url for agent
async fn agent_url_handler(
    sandbox: Arc<dyn Sandbox>,
    _req: Request<Body>,
) -> Result<Response<Body>> {
    let agent_sock = sandbox
        .agent_sock()
        .await
        .unwrap_or_else(|_| String::from(""));
    Ok(Response::new(Body::from(agent_sock)))
}

// ERROR MUST BE WRITTEN TO RESPONSE BODY
async fn direct_volume_stats_handler(
    sandbox: Arc<dyn Sandbox>,
    req: Request<Body>,
) -> Result<Response<Body>> {
    let params = Url::parse(&req.uri().to_string())
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect::<std::collections::HashMap<String, String>>();
    let path = params.get(DIRECT_VOLUME_PATH_KEY).unwrap();
    let path = base64_url::unescape(path).to_string();
    let result = sandbox.guest_volume_stats(&path).await;
    match result {
        Ok(stats) => Ok(Response::new(Body::from(stats))),
        Err(e) => Ok(Response::new(Body::from(e.to_string()))),
    }
}

async fn direct_volume_resize_handler(
    sandbox: Arc<dyn Sandbox>,
    req: Request<Body>,
) -> Result<Response<Body>> {
    let body = hyper::body::to_bytes(req.into_body()).await?;
    // unserialize json body into resizeRequest struct
    let resize_req: ResizeRequest = serde_json::from_slice(&body).unwrap();
    let result = sandbox
        .resize_guest_volume(&resize_req.volume_path, resize_req.size)
        .await;
    match result {
        Ok(_) => Ok(Response::new(Body::from(""))),
        Err(e) => Ok(Response::new(Body::from(format!("resize failed: {}", e)))),
    }
}
