// Auth API client. Validates player tokens against central auth service.

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

#[derive(Debug, Serialize)]
struct ValidateRequest {
    token: String,
}

#[derive(Debug, Deserialize)]
struct ValidateResponse {
    status: String,
    nickname: Option<String>,
}

/// Result of token validation.
#[derive(Debug)]
pub enum AuthResult {
    /// Token valid, contains player nickname.
    Ok(String),
    /// Token invalid or expired.
    Invalid,
    /// Auth service unreachable.
    ServiceError(String),
}

/// Validate a player's auth token against the central auth API.
pub async fn validate_token(auth_url: &str, token: &str) -> AuthResult {
    let url = format!("{}/auth/validate-token", auth_url);
    let client = reqwest::Client::new();

    let body = ValidateRequest {
        token: token.to_string(),
    };

    match client.post(&url).json(&body).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<ValidateResponse>().await {
                    Ok(data) => {
                        if data.status == "ok" {
                            if let Some(nickname) = data.nickname {
                                debug!("Auth success for {}", nickname);
                                return AuthResult::Ok(nickname);
                            }
                        }
                        AuthResult::Invalid
                    }
                    Err(e) => {
                        warn!("Auth response parse error: {}", e);
                        AuthResult::ServiceError(e.to_string())
                    }
                }
            } else {
                debug!("Auth rejected: status {}", response.status());
                AuthResult::Invalid
            }
        }
        Err(e) => {
            warn!("Auth service unreachable: {}", e);
            AuthResult::ServiceError(e.to_string())
        }
    }
}
