use std::sync::LazyLock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shared::{
    ENV_VARIABLE_NAME_SERVER_URL,
    ApiError,
    ChangePasswordRequest,
    ForgotPasswordRequest,
    LoginRequest,
    PasswordResetRequest,
    TokenCheckRequest,
    User
};

// Not browser
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{Client, Method};

// Browser
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use web_sys::RequestCredentials;

const URL_SCHEME: &str = "http://";

// Base URL for the API, read from the environment variable or defaulting to localhost:3000
static API_BASE: LazyLock<String> = LazyLock::new(|| {
    std::env::var(ENV_VARIABLE_NAME_SERVER_URL).unwrap_or_else(|_| "127.0.0.1:3000".into())
});


#[cfg(not(target_arch = "wasm32"))]
static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build native reqwest client")
});

// Not browser
#[cfg(not(target_arch = "wasm32"))]
const GET_METHOD: Method = Method::GET;
#[cfg(not(target_arch = "wasm32"))]
const POST_METHOD: Method = Method::POST;

// Browser
#[cfg(target_arch = "wasm32")]
const GET_METHOD: &str = "GET";
#[cfg(target_arch = "wasm32")]
const POST_METHOD: &str = "POST";

// Not browser
#[cfg(not(target_arch = "wasm32"))]
async fn request_json<T: Serialize + ?Sized, R: DeserializeOwned>(
    method: Method,
    url: String,
    body: Option<&T>,
) -> Result<R, String> {
    let request = CLIENT.request(method, url);
    let request = if let Some(body) = body {
        request.json(body)
    } else {
        request
    };

    let resp = request.send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(err.message);
        }
        return Err(format!("Request failed with status {}: {}", status, body));
    }

    serde_json::from_str::<R>(&body)
        .map_err(|e| {
            if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
                err.message
            } else {
                e.to_string()
            }
        })
}

// Browser
#[cfg(target_arch = "wasm32")]
async fn request_json<T: Serialize + ?Sized, R: DeserializeOwned>(
    method: &str,
    url: String,
    body: Option<&T>,
) -> Result<R, String> {
    let request_builder = match method {
        POST_METHOD => Request::post(&url),
        _ => Request::get(&url),
    }
    .credentials(RequestCredentials::Include);

    let resp = if let Some(body) = body {
        let request = request_builder
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(body).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        request.send().await.map_err(|e| e.to_string())?
    } else {
        request_builder.send().await.map_err(|e| e.to_string())?
    };

    // capture ok/status before consuming body
    let ok = resp.ok();
    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;

    if !ok {
        if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
            return Err(err.message);
        }
        return Err(format!("Request failed with status {}: {}", status, body));
    }

    serde_json::from_str::<R>(&body)
        .map_err(|e| {
            if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
                err.message
            } else {
                e.to_string()
            }
        })
}


// Function to use existing user session
pub async fn me() -> Option<User> {
    request_json::<(), User>(
        GET_METHOD,
        format!("{}{}/me", URL_SCHEME, *API_BASE),
        None,
    )
    .await
    .ok()
}


// Function to log in the user
pub async fn login(req: LoginRequest) -> Option<User> {
    request_json(
        POST_METHOD,
        format!("{}{}/login", URL_SCHEME, *API_BASE),
        Some(&req),
    )
    .await
    .ok()
}


// Function to log out the user
pub async fn logout() -> Option<bool> {
    request_json::<(), bool>(
        POST_METHOD,
        format!("{}{}/logout", URL_SCHEME, *API_BASE),
        None,
    )
    .await
    .ok()
}


// Function to get a list of all users
pub async fn list_users() -> Result<Vec<User>, String> {
    request_json::<(), Vec<User>>(
        GET_METHOD,
        format!("{}{}/users/list", URL_SCHEME, *API_BASE),
        None,
    ).await
}


// Function to save a user (create or update)
pub async fn save_user(user: &User) -> Option<bool> {
    request_json(
        POST_METHOD,
        format!("{}{}/users/save", URL_SCHEME, *API_BASE),
        Some(user),
    )
    .await
    .ok()
}


// Function to delete a user by ID
pub async fn delete_user(user_id: i32) -> Option<bool> {
    request_json(
        POST_METHOD,
        format!("{}{}/users/delete", URL_SCHEME, *API_BASE),
        Some(&user_id),
    )
    .await
    .ok()
}


// Function for an admin to request a user password reset
pub async fn admin_request_user_password_reset(user: User) -> Result<bool, String> {
    request_json(
        POST_METHOD,
        format!("{}{}/users/admin_request_password_reset", URL_SCHEME, *API_BASE),
        Some(&user),
    )
    .await
}


// Function for a user to request a password reset
pub async fn user_request_user_password_reset(req: ForgotPasswordRequest) -> Result<bool, String> {
    request_json(
        POST_METHOD,
        format!("{}{}/users/user_request_password_reset", URL_SCHEME, *API_BASE),
        Some(&req),
    )
    .await
}


// Function to reset a user's password
pub async fn check_reset_token(req: TokenCheckRequest) -> Option<User> {
    request_json(
        POST_METHOD,
        format!("{}{}/users/check_reset_token", URL_SCHEME, *API_BASE),
        Some(&req),
    )
    .await
    .ok()
}


// Function to reset a user's password
pub async fn reset_user_password(req: PasswordResetRequest) -> Result<bool, String> {
    request_json(
        POST_METHOD,
        format!("{}{}/users/reset_password", URL_SCHEME, *API_BASE),
        Some(&req),
    )
    .await
}


// Function to change a user's password
pub async fn change_user_password(req: &ChangePasswordRequest) -> Result<bool, String> {
    request_json(
        POST_METHOD,
        format!("{}{}/users/change_password", URL_SCHEME, *API_BASE),
        Some(req),
    )
    .await
}


//-------------------------------------------------------------
// Unit tests
//-------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_api_error_into_string_when_expected_bool() {
        let body = r#"{"message":"Password too weak"}"#;

        // simulate the parsing logic used in request_json: try to parse R (bool),
        // on failure try to parse ApiError and return its message
        let result: Result<bool, String> = serde_json::from_str::<bool>(body).map_err(|e| {
            if let Ok(err) = serde_json::from_str::<ApiError>(body) {
                err.message
            } else {
                e.to_string()
            }
        });

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Password too weak");
    }
}
