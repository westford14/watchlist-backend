use crate::api::error::APIError;
use crate::api::version::APIVersion;
use crate::domain::models::healthz::HealthCheckResponse;
use axum::{Json, response::IntoResponse};

pub async fn health_check(api_version: APIVersion) -> Result<impl IntoResponse, APIError> {
    tracing::trace!("api version: {}", api_version);
    let json_response = serde_json::json!(HealthCheckResponse {
        status: 200,
        message: "healthy".to_string()
    });

    Ok(Json(json_response))
}
