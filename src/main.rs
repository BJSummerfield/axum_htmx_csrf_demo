use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, post},
    BoxError, Router,
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

mod auth;
mod handlers;
mod models;
mod render;

use auth::Auth;
use handlers::Handle;

#[tokio::main]
async fn main() {
    let session_store = MemoryStore::default();
    let csrf_config = CsrfConfig::default();
    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(true)
                .with_expiry(Expiry::OnSessionEnd),
        )
        .layer(CsrfLayer::new(csrf_config));

    let app = Router::new()
        .route("/username", post(Handle::username))
        .layer(axum::middleware::from_fn(Auth::csrf_middleware))
        .route("/", get(Handle::root))
        .layer(session_service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
