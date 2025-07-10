use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use sqlx::types::Uuid;
use thiserror::Error;

use crate::{
    api::error::{API_DOCUMENT_URL, APIError, APIErrorCode, APIErrorEntry, APIErrorKind},
    api::version::{self, APIVersion},
    application::{
        repository::movie_repo,
        security::jwt::{AccessClaims, ClaimsMethods},
        state::SharedState,
    },
    domain::models::movie::{Movie, PaginatedResponse, PaginationParams},
};

pub async fn list_movies_by_user_handler(
    api_version: APIVersion,
    access_claims: AccessClaims,
    State(state): State<SharedState>,
    Json(pagination): Json<PaginationParams>,
) -> Result<Json<PaginatedResponse>, APIError> {
    tracing::trace!("api version: {}", api_version);
    tracing::trace!("authentication details: {:#?}", access_claims);
    access_claims.validate_role_admin()?;
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(25);
    let offset = (page - 1) * per_page;
    let total_movies = movie_repo::list_movie_length(&state).await?;

    let movies = movie_repo::list_paginated(
        pagination.username,
        pagination.runtime,
        per_page,
        offset,
        &state,
    )
    .await?;
    Ok(Json(PaginatedResponse {
        page,
        per_page,
        total: total_movies,
        data: movies,
    }))
}

pub async fn list_movies_handler(
    api_version: APIVersion,
    access_claims: AccessClaims,
    State(state): State<SharedState>,
) -> Result<Json<Vec<Movie>>, APIError> {
    tracing::trace!("api version: {}", api_version);
    tracing::trace!("authentication details: {:#?}", access_claims);
    access_claims.validate_role_admin()?;
    let movies = movie_repo::list(&state).await?;
    Ok(Json(movies))
}

pub async fn get_movie_handler(
    access_claims: AccessClaims,
    Path((version, id)): Path<(String, Uuid)>,
    State(state): State<SharedState>,
) -> Result<Json<Movie>, APIError> {
    let api_version: APIVersion = version::parse_version(&version)?;
    tracing::trace!("api version: {}", api_version);
    tracing::trace!("authentication details: {:#?}", access_claims);
    tracing::trace!("id: {}", id);
    access_claims.validate_role_admin()?;
    let movie = movie_repo::get_by_id(id, &state)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                let user_error = MovieError::MovieNotFound(id);
                (user_error.status_code(), APIErrorEntry::from(user_error)).into()
            }
            _ => APIError::from(e),
        })?;

    Ok(Json(movie))
}

pub async fn add_movie_handler(
    api_version: APIVersion,
    access_claims: AccessClaims,
    State(state): State<SharedState>,
    Json(mut movie): Json<Movie>,
) -> Result<impl IntoResponse, APIError> {
    tracing::trace!("api version: {}", api_version);
    tracing::trace!("authentication details: {:#?}", access_claims);
    access_claims.validate_role_admin()?;
    let naive_now = Utc::now().naive_utc();
    movie.created_at = Some(naive_now);
    movie.updated_at = Some(naive_now);
    let movie = movie_repo::add(movie, &state).await?;
    Ok((StatusCode::CREATED, Json(movie)))
}

pub async fn update_movie_handler(
    access_claims: AccessClaims,
    Path((version, id)): Path<(String, Uuid)>,
    State(state): State<SharedState>,
    Json(movie): Json<Movie>,
) -> Result<Json<Movie>, APIError> {
    let api_version: APIVersion = version::parse_version(&version)?;
    tracing::trace!("api version: {}", api_version);
    tracing::trace!("authentication details: {:#?}", access_claims);
    tracing::trace!("id: {}", id);
    let movie = movie_repo::update(movie, &state).await?;
    Ok(Json(movie))
}

pub async fn delete_movie_handler(
    access_claims: AccessClaims,
    Path((version, id)): Path<(String, Uuid)>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, APIError> {
    let api_version: APIVersion = version::parse_version(&version)?;
    tracing::trace!("api version: {}", api_version);
    tracing::trace!("authentication details: {:#?}", access_claims);
    tracing::trace!("id: {}", id);
    access_claims.validate_role_admin()?;
    if movie_repo::delete(id, &state).await? {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)?
    }
}

#[derive(Debug, Error)]
enum MovieError {
    #[error("movie not found: {0}")]
    MovieNotFound(Uuid),
}

impl MovieError {
    const fn status_code(&self) -> StatusCode {
        match self {
            Self::MovieNotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl From<MovieError> for APIErrorEntry {
    fn from(movie_error: MovieError) -> Self {
        let message = movie_error.to_string();
        match movie_error {
            MovieError::MovieNotFound(movie_id) => Self::new(&message)
                .code(APIErrorCode::UserNotFound)
                .kind(APIErrorKind::ResourceNotFound)
                .description(&format!("movie with the ID '{}' does not exist in our records", movie_id))
                .detail(serde_json::json!({"movie_id": movie_id}))
                .reason("must be an existing user")
                .instance(&format!("/api/v1/movie/{}", movie_id))
                .trace_id()
                .help(&format!("please check if the user ID is correct or refer to our documentation at {}#errors for more information", API_DOCUMENT_URL))
                .doc_url(),
        }
    }
}
