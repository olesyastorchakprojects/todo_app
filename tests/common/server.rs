use axum::Router;
use reqwest::Url;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct TestAppHandle {
    pub address: Url,
    pub _shutdown: oneshot::Sender<()>,
    pub _server_task: JoinHandle<()>,
}

pub async fn spawn_test_app(app: Router) -> TestAppHandle {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let server = axum::serve(listener, app.into_make_service()).with_graceful_shutdown(async {
        shutdown_rx.await.ok();
    });

    let server_task = tokio::spawn(async move {
        if let Err(e) = server.await {
            println!("server error : {:?}", e);
        }
    });

    TestAppHandle {
        address: Url::parse(&format!("http://{}", addr)).unwrap(),
        _shutdown: shutdown_tx,
        _server_task: server_task,
    }
}
