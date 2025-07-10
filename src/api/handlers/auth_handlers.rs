use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use bcrypt::verify;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::Uuid;

use crate::{
    api::error::{APIError, APIErrorCode, APIErrorEntry, APIErrorKind},
    api::version::APIVersion,
    application::{
        repository::user_repo,
        security::{
            auth::{self, AuthError, JwtTokens},
            jwt::{AccessClaims, ClaimsMethods, RefreshClaims},
        },
        state::SharedState,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginUser {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeUser {
    user_id: Uuid,
}

#[tracing::instrument(level = tracing::Level::TRACE, name = "login", skip_all, fields(username=login.username))]
pub async fn login_handler(
    api_version: APIVersion,
    State(state): State<SharedState>,
    Json(login): Json<LoginUser>,
) -> Result<impl IntoResponse, APIError> {
    tracing::trace!("api version: {}", api_version);
    if let Ok(user) = user_repo::get_by_username(&login.username, &state).await {
        let is_valid =
            verify(login.password, &user.password_hash).expect("Failed to verify password");
        if is_valid {
            tracing::trace!("access granted, user: {}", user.id);
            let tokens = auth::generate_tokens(user, &state.config);
            let response = tokens_to_response(tokens);
            return Ok(response);
        }
    }
    Err(AuthError::WrongCredentials)?
}

pub async fn logout_handler(
    api_version: APIVersion,
    State(state): State<SharedState>,
    refresh_claims: RefreshClaims,
) -> Result<impl IntoResponse, APIError> {
    tracing::trace!("api version: {}", api_version);
    tracing::trace!("refresh_claims: {:?}", refresh_claims);
    auth::logout(refresh_claims, state).await?;
    Ok(())
}

pub async fn cleanup_handler(
    api_version: APIVersion,
    State(state): State<SharedState>,
    access_claims: AccessClaims,
) -> Result<impl IntoResponse, APIError> {
    tracing::trace!("api version: {}", api_version);
    access_claims.validate_role_admin()?;
    tracing::trace!("authentication details: {:#?}", access_claims);
    let deleted = auth::cleanup_revoked_and_expired(&access_claims, &state).await?;
    let json = json!({
        "deleted_tokens": deleted,
    });
    Ok(Json(json))
}

fn tokens_to_response(jwt_tokens: JwtTokens) -> impl IntoResponse {
    let json = json!({
        "access_token": jwt_tokens.access_token,
        "refresh_token": jwt_tokens.refresh_token,
        "token_type": "Bearer"
    });

    tracing::trace!("JWT: generated response {:#?}", json);
    Json(json)
}

impl From<AuthError> for APIError {
    fn from(auth_error: AuthError) -> Self {
        let (status_code, code) = match auth_error {
            AuthError::WrongCredentials => (
                StatusCode::UNAUTHORIZED,
                APIErrorCode::AuthenticationWrongCredentials,
            ),
            AuthError::MissingCredentials => (
                StatusCode::BAD_REQUEST,
                APIErrorCode::AuthenticationMissingCredentials,
            ),
            AuthError::TokenCreationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                APIErrorCode::AuthenticationTokenCreationError,
            ),
            AuthError::InvalidToken => (
                StatusCode::BAD_REQUEST,
                APIErrorCode::AuthenticationInvalidToken,
            ),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, APIErrorCode::AuthenticationForbidden),
            AuthError::RevokedTokensInactive => (
                StatusCode::BAD_REQUEST,
                APIErrorCode::AuthenticationRevokedTokensInactive,
            ),
            AuthError::RedisError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, APIErrorCode::RedisError)
            }
            AuthError::SQLxError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                APIErrorCode::DatabaseError,
            ),
        };

        let error = APIErrorEntry::new(&auth_error.to_string())
            .code(code)
            .kind(APIErrorKind::AuthenticationError);

        Self {
            status: status_code.as_u16(),
            errors: vec![error],
        }
    }
}
