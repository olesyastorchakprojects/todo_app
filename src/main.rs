use std::net::SocketAddr;

use todo_app::{MetricsProviderGuard, Settings, StartupError, TracingProviderGuard};

use thiserror::Error;
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::signal;
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug, Error)]
enum RoutingAppError {
    #[error("Startup error")]
    Startup(#[from] StartupError),

    #[error("Io error")]
    Io(#[from] std::io::Error),
}

fn main() -> Result<(), RoutingAppError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .max_blocking_threads(num_cpus::get() * 2)
        .enable_all()
        .build()?;

    runtime.block_on(async_main())
}

#[cfg(unix)]
async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to bind to SIGTERM");

    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("SIGINT received.");
        },
        _ = sigterm.recv() => {
            println!("SIGTERM received.");
        },
    }
}

async fn async_main() -> Result<(), RoutingAppError> {
    let settings = Settings::new()?;
    //std::env::set_var("RUST_LOG", "sled=trace");

    let _tracing_provider_guard = settings
        .tracing_enabled()
        .then(|| TracingProviderGuard::new(&settings))
        .transpose()?;

    let _metrics_provider_guard = settings
        .metrics_enabled()
        .then(|| MetricsProviderGuard::new(&settings))
        .transpose()?;

    let server_addr = settings.server_addr();
    let (app, service) = todo_app::init_app(settings).await?;

    let listener = TcpListener::bind(&server_addr).await?;

    let shutdown_signal = async {
        #[cfg(unix)]
        shutdown_signal().await;
    };

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal)
    .await?;

    let _ = service.flush_storage().await;

    Ok(())
}
