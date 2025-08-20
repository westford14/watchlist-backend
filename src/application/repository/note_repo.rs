use chrono::Utc;
use sqlx::query_as;

use crate::{
    application::{repository::RepositoryResult, state::SharedState},
    domain::models::note::Note,
};

pub async fn list(state: &SharedState) -> RepositoryResult<Vec<Note>> {
    let users = query_as::<_, Note>("SELECT * FROM notes")
        .fetch_all(&state.db_pool)
        .await?;

    Ok(users)
}

pub async fn add(note: Note, state: &SharedState) -> RepositoryResult<Note> {
    let time_now = Utc::now().naive_utc();
    tracing::trace!("note: {:#?}", note);
    let note = sqlx::query_as::<_, Note>(
        r#"INSERT INTO notes (id,
         username,
         tmdb_id,
         note,
         created_at,
         updated_at)
         VALUES ($1,$2,$3,$4,$5,$6)
         RETURNING notes.*"#,
    )
    .bind(note.id)
    .bind(note.username)
    .bind(note.tmdb_id)
    .bind(note.note)
    .bind(time_now)
    .bind(time_now)
    .fetch_one(&state.db_pool)
    .await?;

    Ok(note)
}

pub async fn get_by_id(id: i32, state: &SharedState) -> RepositoryResult<Note> {
    let note = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE tmdb_id = $1")
        .bind(id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(note)
}

pub async fn get_by_username(username: &str, state: &SharedState) -> RepositoryResult<Vec<Note>> {
    let note = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE username = $1")
        .bind(username)
        .fetch_all(&state.db_pool)
        .await?;

    Ok(note)
}

pub async fn update(note: Note, state: &SharedState) -> RepositoryResult<Note> {
    tracing::trace!("note: {:#?}", note);
    let time_now = Utc::now().naive_utc();
    let note = sqlx::query_as::<_, Note>(
        r#"UPDATE notes
         SET 
         note = $1,
         updated_at = $2
         WHERE tmdb_id = $3
         RETURNING notes.*"#,
    )
    .bind(note.note)
    .bind(time_now)
    .bind(note.tmdb_id)
    .fetch_one(&state.db_pool)
    .await?;

    Ok(note)
}

pub async fn delete(id: i32, state: &SharedState) -> RepositoryResult<bool> {
    let query_result = sqlx::query("SELECT * FROM notes WHERE tmdb_id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;

    Ok(query_result.rows_affected() == 1)
}
