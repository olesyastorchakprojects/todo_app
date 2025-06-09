pub(crate) mod blocking_task_guard;
pub(crate) mod measure_metrics;
pub(crate) mod metrics;
pub(crate) mod root_span;

pub(crate) static JWT_SECRET_KEY: &str = "JWT_SECRET";
pub(crate) static APP_NAME: &str = "todo_app";

pub(crate) use root_span::RootSpan;

#[cfg(feature = "jemalloc")]
mod jemalloc_metrics;

#[macro_export]
macro_rules! trace_err {
    ($expr:expr, $($arg:tt)*) => {
        $expr.map_err(|e| {
            ::tracing::error!(%e, $($arg)*);
            e
        })
    };
}
