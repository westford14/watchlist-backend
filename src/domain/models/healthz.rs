use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthCheckResponse {
    pub status: i16,
    pub message: String,
}
