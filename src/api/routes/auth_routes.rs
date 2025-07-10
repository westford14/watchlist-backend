use axum::{Router, routing::post};

use crate::{
    api::handlers::auth_handlers::{cleanup_handler, login_handler, logout_handler},
    application::state::SharedState,
};

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/cleanup", post(cleanup_handler))
}
