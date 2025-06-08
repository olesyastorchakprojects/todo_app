use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
};

use opentelemetry::KeyValue;
use tokio::time::Instant;
use tracing::{info, info_span, instrument};

use crate::{middleware::normalize_uri, utils::metrics};

#[instrument(name = "middleware::record_metrics", skip_all)]
pub(crate) async fn record_metrics(
    request: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    info!(?request, "record_metrics middleware");

    let method = request.method().to_string();
    let uri = normalize_uri(&request.uri().to_string());

    let start = Instant::now();

    let response = next.run(request).await;

    let elapsed = start.elapsed();

    let status = response.status();
    let status_str = status.as_str().to_string();
    let status_group = match status.as_u16() {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "unknown",
    };

    tracing::info!(?elapsed, %method, %uri, ?status, %status_str, %status_group, "record_metrics: after next.run()");

    info_span!("record metric REQUEST_COUNTER").in_scope(|| {
        metrics::REQUEST_COUNTER.add(
            1.0,
            &[
                KeyValue::new("method", method.clone()),
                KeyValue::new("uri", uri.clone()),
                KeyValue::new("status_group", status_group),
                KeyValue::new("http_status_code", status_str.clone()),
            ],
        )
    });

    info_span!("record metric HTTP_REQUEST_DURATION_HISTOGRAM").in_scope(|| {
        metrics::HTTP_REQUEST_DURATION_HISTOGRAM.record(
            elapsed.as_millis() as f64,
            &[
                KeyValue::new("method", method),
                KeyValue::new("uri", uri),
                KeyValue::new("status_group", status_group),
                KeyValue::new("http_status_code", status_str),
            ],
        )
    });

    Ok(response)
}
