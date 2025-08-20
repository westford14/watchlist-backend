use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    api::handlers::note_handlers::{
        add_note_handler, delete_note_handler, get_note_handler, list_notes_handler,
        update_note_handler,
    },
    application::state::SharedState,
};

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/", get(list_notes_handler))
        .route("/", post(add_note_handler))
        .route("/{id}", get(get_note_handler))
        .route("/{id}", post(update_note_handler))
        .route("/{id}", delete(delete_note_handler))
}
