use super::APP_NAME;
use jemalloc_ctl::{epoch, stats};
use opentelemetry::KeyValue;

pub(crate) fn init_jemalloc_metrics() {
    let meter = opentelemetry::global::meter(APP_NAME);

    let _gauge_allocated = meter
        .f64_observable_gauge("jemalloc_allocated_bytes")
        .with_description("Bytes allocated by jemalloc (malloc)")
        .with_unit("bytes")
        .with_callback(move |observer| {
            let _ = epoch::advance();
            if let Ok(allocated) = stats::allocated::read() {
                observer.observe(allocated as f64, &[KeyValue::new("allocator", "jemalloc")]);
            }
        })
        .build();

    let _gauge_active = meter
        .f64_observable_gauge("jemalloc_active_bytes")
        .with_description("Bytes actively used by jemalloc")
        .with_unit("bytes")
        .with_callback(move |observer| {
            let _ = epoch::advance();
            if let Ok(active) = stats::active::read() {
                observer.observe(active as f64, &[KeyValue::new("allocator", "jemalloc")]);
            }
        })
        .build();

    let _gauge_resident = meter
        .f64_observable_gauge("jemalloc_resident_bytes")
        .with_description("Bytes resident in memory according to jemalloc")
        .with_unit("bytes")
        .with_callback(move |observer| {
            let _ = epoch::advance();
            if let Ok(resident) = stats::resident::read() {
                observer.observe(resident as f64, &[KeyValue::new("allocator", "jemalloc")]);
            }
        })
        .build();
}
