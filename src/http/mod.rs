use axum::{Router, extract::State, http::StatusCode, routing::get};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
pub struct HttpState {
    ready: Arc<std::sync::atomic::AtomicBool>,
}

impl HttpState {
    pub fn new() -> Self {
        Self {
            ready: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for HttpState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn router(state: HttpState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn serve(addr: SocketAddr, state: HttpState) -> Result<(), std::io::Error> {
    let listener = TcpListener::bind(addr).await?;
    info!(%addr, "starting http server");
    axum::serve(listener, router(state)).await
}

async fn healthz() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn readyz(State(state): State<HttpState>) -> StatusCode {
    if state.is_ready() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn metrics() -> &'static str {
    "# metrics exporter not configured yet\n"
}
