use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Uuid};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub username: String,
    pub runtime: i64,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct PaginatedResponse {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub data: Vec<Movie>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, PartialEq, Clone)]
pub struct Movie {
    pub id: Uuid,
    pub name: String,
    pub letterboxd_id: i32,
    pub url: String,
    pub tmdb_id: i32,
    pub username: String,
    pub runtime: i32,
    pub poster_path: String,
    pub vote_average: f64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
