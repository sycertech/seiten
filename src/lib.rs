use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, MatchedPath},
    http::{HeaderName, Request},
    routing::post,
    BoxError, Router,
};
use http::StatusCode;
use request_id::MyRequestId;
use routes::upload;
use std::time::Duration;
use tower::{
    timeout::{error::Elapsed, TimeoutLayer},
    ServiceBuilder,
};
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info_span;

pub mod config;
pub mod request_id;
pub mod routes;

pub fn create_router() -> Router<()> {
    let x_request_id = HeaderName::from_static("x-request-id");

    let request_id_layer = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MyRequestId::default(),
        ))
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(MatchedPath::as_str);

        info_span!(
            "http_request",
            method = ?request.method(),
            matched_path,
        )
    });

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            if e.is::<Elapsed>() {
                (StatusCode::REQUEST_TIMEOUT, e.to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let middleware = ServiceBuilder::new()
        .layer(request_id_layer)
        .layer(trace_layer)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 50)) // 50 MiB
        .layer(timeout_layer);

    Router::new()
        .route("/upload/:outdir", post(upload))
        .layer(middleware)
}
