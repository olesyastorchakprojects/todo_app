use std::future::Future;

use super::metrics::{self};

use opentelemetry::KeyValue;
use tokio::time::Instant;
use tracing::error;

pub async fn measure_and_record_service<F, T, E>(
    operation_name: &'static str,
    f: impl FnOnce() -> F,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let start = Instant::now();
    let result = f().await;
    let elapsed = start.elapsed().as_millis() as f64;

    let (status, error_kind) = match &result {
        Err(e) => ("error", format!("error: {}", e)),
        Ok(_) => ("ok", "NA".to_string()),
    };

    metrics::SERVICE_OPERATION_DURATION_HISTOGRAM.record(
        elapsed,
        &[
            KeyValue::new("operation", operation_name),
            KeyValue::new("status", status),
            KeyValue::new("error_kind", error_kind),
        ],
    );

    result
}

pub fn measure_and_record_storage<T, E>(
    operation_name: &'static str,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<T, E>
where
    E: std::fmt::Display,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed().as_millis() as f64;

    let (status, err_kind) = match &result {
        Err(e) => ("error", format!("error: {}", e)),
        Ok(_) => ("ok", "NA".to_string()),
    };

    metrics::STORAGE_OPERATION_DURATION_HISTOGRAM.record(
        elapsed,
        &[
            KeyValue::new("operation", operation_name),
            KeyValue::new("storage", "sled"),
            KeyValue::new("status", status),
            KeyValue::new("err_kind", err_kind),
        ],
    );

    result
}

pub fn measure_and_record_password_hash<T, E>(
    algorithm: &'static str,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<T, E>
where
    E: AsRef<str>,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed().as_millis() as f64;

    let (status, err_kind) = match &result {
        Err(e) => ("error", e.as_ref().to_string()),
        Ok(_) => ("ok", "NA".to_string()),
    };

    metrics::PASSWORD_HASH_TIME_HISTOGRAM.record(
        elapsed,
        &[
            KeyValue::new("algorithm", algorithm),
            KeyValue::new("status", status),
            KeyValue::new("err_kind", err_kind),
        ],
    );

    result
}

pub fn measure_and_record_password_verify<T, E>(
    algorithm: &'static str,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<T, E>
where
    E: AsRef<str>,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed().as_millis() as f64;

    let (status, err_kind) = match &result {
        Err(e) => ("error", e.as_ref().to_string()),
        Ok(_) => ("ok", "NA".to_string()),
    };

    metrics::PASSWORD_VERIFY_TIME_HISTOGRAM.record(
        elapsed,
        &[
            KeyValue::new("algorithm", algorithm),
            KeyValue::new("status", status),
            KeyValue::new("err_kind", err_kind),
        ],
    );

    result
}

pub(crate) fn init_memory_metrics() {
    init_rss_metrics();

    #[cfg(all(feature = "jemalloc"))]
    super::jemalloc_metrics::init_jemalloc_metrics();
}

fn init_rss_metrics() {
    opentelemetry::global::meter_provider()
        .meter("todo_app")
        .f64_observable_gauge("process_resident_memory_bytes")
        .with_description("Resident memory size in bytes")
        .with_unit("bytes")
        .with_callback(move |observer| {
            let mut system = sysinfo::System::new_all();
            let current_pid = match sysinfo::get_current_pid() {
                Ok(pid) => pid,
                Err(err) => {
                    error!("Failed to get current PID: {:?}", err);
                    return;
                }
            };
            system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[current_pid]), false);
            if let Some(process) = system.process(current_pid) {
                let mem_bytes = (process.memory() as f64) * 1024.0; // KB → bytes
                observer.observe(mem_bytes, &[]);
            } else {
                error!("Failed to get current process by pid");
            }
        })
        .build();
}
