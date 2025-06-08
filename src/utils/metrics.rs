use super::APP_NAME;
use once_cell::sync::Lazy;
use opentelemetry::{
    global::{self},
    metrics::{Counter, Histogram},
};

pub static REQUEST_COUNTER: Lazy<Counter<f64>> = Lazy::new(|| {
    global::meter_provider()
        .meter(APP_NAME)
        .f64_counter("http_requests_total")
        .build()
});

pub static REQUEST_COUNTER_429: Lazy<Counter<f64>> = Lazy::new(|| {
    global::meter_provider()
        .meter(APP_NAME)
        .f64_counter("http_status_code_429_total")
        .build()
});

pub static HTTP_REQUEST_DURATION_HISTOGRAM: Lazy<Histogram<f64>> = Lazy::new(|| {
    global::meter_provider()
        .meter(APP_NAME)
        .f64_histogram("http_request_duration_milliseconds")
        .build()
});

pub static STORAGE_OPERATION_DURATION_HISTOGRAM: Lazy<Histogram<f64>> = Lazy::new(|| {
    global::meter_provider()
        .meter(APP_NAME)
        .f64_histogram("storage_operation_duration_milliseconds")
        .build()
});

pub static SERVICE_OPERATION_DURATION_HISTOGRAM: Lazy<Histogram<f64>> = Lazy::new(|| {
    global::meter_provider()
        .meter(APP_NAME)
        .f64_histogram("service_operation_duration_milliseconds")
        .build()
});

pub static PASSWORD_HASH_TIME_HISTOGRAM: Lazy<Histogram<f64>> = Lazy::new(|| {
    global::meter_provider()
        .meter(APP_NAME)
        .f64_histogram("password_hash_duration_milliseconds")
        .build()
});

pub static PASSWORD_VERIFY_TIME_HISTOGRAM: Lazy<Histogram<f64>> = Lazy::new(|| {
    global::meter_provider()
        .meter(APP_NAME)
        .f64_histogram("password_verify_duration_milliseconds")
        .build()
});
