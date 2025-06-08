use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
};

use crate::{
    middleware::normalize_uri,
    utils::{root_span::SamplingPriority, RootSpan},
};

pub(crate) async fn trace_root(mut req: Request, next: Next) -> Result<Response<Body>, StatusCode> {
    let root_span = RootSpan::new(
        req.method().as_str(),
        &normalize_uri(&req.uri().to_string()),
    );

    req.extensions_mut().insert(root_span.clone());

    let _enter = root_span.enter();
    let resp = next.run(req).await;

    root_span.record().http_status_code(&resp.status());

    if resp.status().is_client_error() || resp.status().is_server_error() {
        root_span
            .record()
            .status("error")
            .sampling_priority(SamplingPriority::One);
    } else {
        root_span.record().status("ok");
    }

    Ok(resp)
}
