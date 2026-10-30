use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request payload for internal token verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

/// Authenticated user session context returned by token verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyTokenResponse {
    /// Unique user identifier.
    pub user_id: Uuid,
    /// Primary user email address.
    pub email: String,
}

/// Helper for Gateway and downstream microservices to verify an access token.
pub async fn verify_token(
    client: &reqwest::Client,
    auth_service_url: &str,
    token: &str,
) -> Result<VerifyTokenResponse, reqwest::Error> {
    let url = format!(
        "{}/api/v1/auth/verify",
        auth_service_url.trim_end_matches('/')
    );
    client
        .post(&url)
        .json(&VerifyTokenRequest {
            token: token.to_string(),
        })
        .send()
        .await?
        .error_for_status()?
        .json::<VerifyTokenResponse>()
        .await
}
