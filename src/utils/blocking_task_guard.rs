use opentelemetry::KeyValue;

use super::APP_NAME;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicI64, Ordering};

static BLOCKING_TASKS: Lazy<DashMap<&'static str, AtomicI64>> = Lazy::new(DashMap::new);

pub struct BlockingTaskGuard {
    name: &'static str,
}

impl BlockingTaskGuard {
    pub fn new(name: &'static str) -> Self {
        BLOCKING_TASKS
            .entry(name)
            .or_insert(AtomicI64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        Self { name }
    }
}

impl Drop for BlockingTaskGuard {
    fn drop(&mut self) {
        if let Some(count) = BLOCKING_TASKS.get(self.name) {
            count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

pub(crate) fn init_blocking_tasks_metric() {
    let meter = opentelemetry::global::meter(APP_NAME);

    let _gauge_blocking_tasks = meter
        .i64_observable_gauge("blocking_tasks_count")
        .with_callback(move |observer| {
            for entry in BLOCKING_TASKS.iter() {
                observer.observe(
                    entry.load(Ordering::Relaxed),
                    &[KeyValue::new("operation", *entry.key())],
                );
            }
        })
        .build();
}
