use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    response::Response,
};
use governor::{DefaultDirectRateLimiter, DefaultKeyedRateLimiter, Quota, RateLimiter};
use opentelemetry::KeyValue;
use std::{
    future::Future,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::{utils::metrics, Settings};

#[derive(Clone)]
pub struct GlobalRateLimitLayer {
    limiter: Arc<DefaultDirectRateLimiter>,
}

impl GlobalRateLimitLayer {
    pub fn new(cell_per_sec: u32, burst: u32) -> Self {
        let quota = Quota::per_second(
            std::num::NonZeroU32::new(cell_per_sec).expect("cells per second must not be zero"),
        )
        .allow_burst(std::num::NonZeroU32::new(burst).expect("burst must not be zero"));

        let limiter = RateLimiter::direct(quota);
        Self {
            limiter: Arc::new(limiter),
        }
    }
}

impl<S> Layer<S> for GlobalRateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    limiter: Arc<DefaultDirectRateLimiter>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RateLimitMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        if self.limiter.check().is_err() {
            let response = Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .body(Body::from("Too many requests"))
                .unwrap();

            metrics::REQUEST_COUNTER_429.add(1.0, &[KeyValue::new("kind", "global")]);

            return Box::pin(async { Ok(response) });
        }

        let fut = self.inner.call(req);
        Box::pin(fut)
    }
}

#[derive(Clone)]
pub struct PerIpRateLimiter {
    limiter: Arc<DefaultKeyedRateLimiter<IpAddr>>,
}

impl PerIpRateLimiter {
    pub fn new(cell_per_sec: u32, burst: u32) -> Self {
        let quota = Quota::per_second(
            std::num::NonZeroU32::new(cell_per_sec).expect("cells per second must not be zero"),
        )
        .allow_burst(std::num::NonZeroU32::new(burst).expect("burst must not be zero"));

        let limiter = RateLimiter::keyed(quota);
        Self {
            limiter: Arc::new(limiter),
        }
    }
}

impl<S> Layer<S> for PerIpRateLimiter {
    type Service = PerIpRateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PerIpRateLimitMiddleware {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

#[derive(Clone)]
pub struct PerIpRateLimitMiddleware<S> {
    inner: S,
    limiter: Arc<DefaultKeyedRateLimiter<IpAddr>>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for PerIpRateLimitMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let settings = req.extensions().get::<Settings>();

        let x_forwarded_for = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<IpAddr>().ok());

        let ip_addr = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.ip());

        let ip_addr = match settings {
            Some(s) if s.rate_limiter.x_forwarded_for => x_forwarded_for,
            _ => ip_addr,
        };

        if let Some(ip_addr) = ip_addr {
            if self.limiter.check_key(&ip_addr).is_err() {
                let response = Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .body(Body::from("Too many requests per IP address"))
                    .unwrap();

                metrics::REQUEST_COUNTER_429.add(1.0, &[KeyValue::new("kind", "per_ip")]);

                return Box::pin(async { Ok(response) });
            }
        }
        let fut = self.inner.call(req);
        Box::pin(fut)
    }
}
