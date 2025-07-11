use std::{sync::Arc, time::SystemTime};

use axum::{
    Json, Router,
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::Utc;
use hyper::Method;
use serde_json::json;
use tokio::{
    net::TcpListener,
    signal::{
        self,
        unix::{self, SignalKind},
    },
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::routes::{auth_routes, movie_routes, user_routes},
    api::{error::APIError, handlers::healthz_handlers},
    application::state::SharedState,
};

pub async fn start(state: SharedState) {
    // Build a CORS layer.
    // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
    // for more details
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::HEAD,
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        //.allow_credentials(true)
        .allow_headers(Any);
    // Build the router.
    let router = Router::new()
        .route("/", get(root_handler))
        .route("/{version}/version", get(version_handler))
        // Health Routes
        .route("/{version}/healthz", get(healthz_handlers::health_check))
        // Auth Routes
        .nest("/{version}/auth", auth_routes::routes())
        // User Routes
        .nest("/{version}/user", user_routes::routes())
        // Movie Routes
        .nest("/{version}/movie", movie_routes::routes())
        .fallback(error_404_handler)
        .with_state(Arc::clone(&state))
        .layer(cors_layer)
        .layer(middleware::from_fn(logging_middleware));

    // Build the listener.
    let addr = state.config.service_socket_addr();
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);

    // Start the API service.
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    tracing::info!("server shutdown successfully.");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        unix::signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("received termination signal, shutting down...");
}

#[tracing::instrument(level = tracing::Level::TRACE, name = "axum", skip_all, fields(method=request.method().to_string(), uri=request.uri().to_string()))]
pub async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    tracing::trace!(
        "received a {} request to {}",
        request.method(),
        request.uri()
    );
    next.run(request).await
}

// Root handler.
pub async fn root_handler() -> Result<impl IntoResponse, APIError> {
    if tracing::enabled!(tracing::Level::TRACE) {
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .as_secs();
        tracing::trace!("timestamp, std::time {}", timestamp);
        tracing::trace!("timestamp, chrono::Utc {}", Utc::now().timestamp() as usize);
    }
    Ok(Json(json!({"message": "Watchlist-Backend!"})))
}

// Version request handler.
pub async fn version_handler() -> Result<impl IntoResponse, APIError> {
    let result = json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    });
    Ok(Json(result))
}

// 404 handler.
pub async fn error_404_handler(request: Request) -> impl IntoResponse {
    tracing::error!("route not found: {:?}", request);
    StatusCode::NOT_FOUND
}
