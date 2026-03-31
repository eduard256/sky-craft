// Auth API client. Handles login flow: request code, verify code, get token.
// Uses reqwest blocking client (called from UI thread).

use serde::{Deserialize, Serialize};

const AUTH_API_URL: &str = "https://apiskycraft.webaweba.com";

#[derive(Debug, Serialize)]
struct RequestCodeBody {
    nickname: String,
}

#[derive(Debug, Serialize)]
struct VerifyCodeBody {
    nickname: String,
    code: String,
    device_info: String,
}

#[derive(Debug, Deserialize)]
struct RequestCodeResponse {
    status: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct VerifyCodeResponse {
    status: String,
    token: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    detail: String,
}

/// Request a login code to be sent to the player's Telegram.
pub fn request_code(nickname: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/auth/request-code", AUTH_API_URL);

    let resp = client
        .post(&url)
        .json(&RequestCodeBody { nickname: nickname.to_string() })
        .send()
        .map_err(|e| format!("Connection error: {}", e))?;

    if resp.status().is_success() {
        let data: RequestCodeResponse = resp.json().map_err(|e| format!("Parse error: {}", e))?;
        Ok(data.message)
    } else {
        let err: ErrorResponse = resp.json().map_err(|e| format!("Parse error: {}", e))?;
        Err(err.detail)
    }
}

/// Verify the code and get a session token.
pub fn verify_code(nickname: &str, code: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/auth/verify-code", AUTH_API_URL);

    let device_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);

    let resp = client
        .post(&url)
        .json(&VerifyCodeBody {
            nickname: nickname.to_string(),
            code: code.to_string(),
            device_info,
        })
        .send()
        .map_err(|e| format!("Connection error: {}", e))?;

    if resp.status().is_success() {
        let data: VerifyCodeResponse = resp.json().map_err(|e| format!("Parse error: {}", e))?;
        Ok(data.token)
    } else {
        let err: ErrorResponse = resp.json().map_err(|e| format!("Parse error: {}", e))?;
        Err(err.detail)
    }
}
