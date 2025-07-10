use chrono::Utc;
use sqlx::query_as;
use uuid::Uuid;

use crate::{
    application::{repository::RepositoryResult, state::SharedState},
    domain::models::movie::Movie,
};

pub async fn list_movie_length(state: &SharedState) -> RepositoryResult<i64> {
    let total_movies: (i64,) = query_as("SELECT COUNT(*) FROM movies")
        .fetch_one(&state.db_pool)
        .await?;

    Ok(total_movies.0)
}

pub async fn list(state: &SharedState) -> RepositoryResult<Vec<Movie>> {
    let users = query_as::<_, Movie>("SELECT * FROM movies")
        .fetch_all(&state.db_pool)
        .await?;

    Ok(users)
}

pub async fn list_paginated(
    username: String,
    runtime: i64,
    limit: i64,
    offset: i64,
    state: &SharedState,
) -> RepositoryResult<Vec<Movie>> {
    let users = query_as::<_, Movie>(
        r#"SELECT * FROM movies
            WHERE runtime <= $1 AND
            username = $2
            ORDER BY vote_average DESC
            LIMIT $3
            OFFSET $4
            "#,
    )
    .bind(runtime)
    .bind(username)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(users)
}

pub async fn list_by_user(username: String, state: &SharedState) -> RepositoryResult<Vec<Movie>> {
    let users = query_as::<_, Movie>("SELECT * FROM movies WHERE username = $1")
        .bind(username)
        .fetch_all(&state.db_pool)
        .await?;

    Ok(users)
}

pub async fn add(movie: Movie, state: &SharedState) -> RepositoryResult<Movie> {
    let time_now = Utc::now().naive_utc();
    tracing::trace!("movie: {:#?}", movie);
    let movie = sqlx::query_as::<_, Movie>(
        r#"INSERT INTO users (id,
         name,
         letterboxd_id,
         url,
         tmdb_id,
         username,
         created_at,
         updated_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
         RETURNING movies.*"#,
    )
    .bind(movie.id)
    .bind(movie.name)
    .bind(movie.letterboxd_id)
    .bind(movie.url)
    .bind(movie.tmdb_id)
    .bind(movie.username)
    .bind(time_now)
    .bind(time_now)
    .fetch_one(&state.db_pool)
    .await?;

    Ok(movie)
}

pub async fn get_by_id(id: Uuid, state: &SharedState) -> RepositoryResult<Movie> {
    let movie = sqlx::query_as::<_, Movie>("SELECT * FROM movies WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db_pool)
        .await?;
    Ok(movie)
}

pub async fn get_by_name(name: &str, state: &SharedState) -> RepositoryResult<Movie> {
    let movie = sqlx::query_as::<_, Movie>("SELECT * FROM movies WHERE name = $1")
        .bind(name)
        .fetch_one(&state.db_pool)
        .await?;

    Ok(movie)
}

pub async fn update(movie: Movie, state: &SharedState) -> RepositoryResult<Movie> {
    tracing::trace!("movie: {:#?}", movie);
    let time_now = Utc::now().naive_utc();
    let movie = sqlx::query_as::<_, Movie>(
        r#"UPDATE movies
         SET 
         name = $1,
         letterboxd_id = $2,
         url = $3,
         tmdb_id = $4,
         username = $5,
         updated_at = $6,
         WHERE id = $7
         RETURNING movies.*"#,
    )
    .bind(movie.name)
    .bind(movie.letterboxd_id)
    .bind(movie.url)
    .bind(movie.tmdb_id)
    .bind(movie.username)
    .bind(time_now)
    .fetch_one(&state.db_pool)
    .await?;

    Ok(movie)
}

pub async fn delete(id: Uuid, state: &SharedState) -> RepositoryResult<bool> {
    let query_result = sqlx::query("SELECT * FROM movies WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await?;

    Ok(query_result.rows_affected() == 1)
}
