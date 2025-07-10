use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    api::handlers::movie_handlers::{
        add_movie_handler, delete_movie_handler, get_movie_handler, list_movies_by_user_handler,
        list_movies_handler, update_movie_handler,
    },
    application::state::SharedState,
};

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_movies_handler))
        .route("/", post(list_movies_by_user_handler))
        .route("/add", post(add_movie_handler))
        .route("/{id}", get(get_movie_handler))
        .route("/{id}", put(update_movie_handler))
        .route("/{id}", delete(delete_movie_handler))
}
