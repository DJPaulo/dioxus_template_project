mod db_sqlite;
mod security;

use std::env;
use std::sync::Arc;
use axum::{
    http,
    routing::{post, get},
    Router,
    extract::{State, FromRequestParts, FromRef},
    body::Body,
    middleware::{from_fn, Next},
    response::Response,
    Json,
};
use secrecy::SecretString;
use tower_http::cors::CorsLayer;
use tower_cookies::{Cookies, Cookie, CookieManagerLayer};
use uuid::Uuid;
use shared::*;



struct AppState {
    db: sqlx::SqlitePool,
}
pub struct AuthUser {
    pub user: User,
    pub session_id: String,
}

// Implement FromRequestParts for AuthUser to extract the authenticated user from the request
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = Json<ApiError>;

    fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // Extract cookies
            let cookies = parts
                .extensions
                .get::<Cookies>()
                .cloned()
                .ok_or_else(|| Json(ApiError { message: "No cookies".into() }))?;

            let session_id = cookies
                .get("session")
                .map(|c| c.value().to_string())
                .ok_or_else(|| Json(ApiError { message: "Not logged in".into() }))?;

            // Extract AppState
            let app_state = Arc::<AppState>::from_ref(state);
            let db = &app_state.db;

            // Query DB
            let session_response = db_sqlite::get_user_by_session(db, &session_id)
                .await
                .map_err(|e| Json(ApiError { message: format!("Database error: {}", e) }))?
                .ok_or_else(|| Json(ApiError { message: "Invalid session".into() }))?;

            Ok(AuthUser {
                user: User {
                    id: session_response.id,
                    username: session_response.username,
                    email: session_response.email,
                    role: session_response.role,
                    is_active: session_response.is_active,
                    change_password: session_response.change_password,
                },
                session_id: session_id.to_string(),
            })
        }
    }
}


impl AuthUser {
    pub fn require_admin(&self) -> Result<(), ApiError> {
        if self.user.role != Role::Admin {
            return Err(ApiError { message: "Forbidden".into() });
        }
        Ok(())
    }

    pub fn require_active(&self) -> Result<(), ApiError> {
        if !self.user.is_active {
            return Err(ApiError { message: "Account inactive".into() });
        }
        Ok(())
    }
}


// Options bypass middleware to avoid CORS issues
async fn options_bypass(req: http::Request<Body>, next: Next) -> Response {
    if req.method() == http::Method::OPTIONS {
        return Response::builder()
            .status(200)
            .body(axum::body::Body::empty())
            .unwrap();
    }
    next.run(req).await
}


#[tokio::main]
async fn main() {
    println!("Setting up DB connection ...");

    let db = db_sqlite::connect_or_create_database().await.expect("Failed to connect to database");
    let state = AppState { db };

    let web_client = &env::var(ENV_VARIABLE_NAME_CLIENT_URL).unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    // Allowed browser client addresses
    let origins = [
        http::HeaderValue::from_str(&web_client).expect("Invalid header value"),
    ];
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::ACCEPT,
        ])
        .allow_credentials(true);

    let login_route = Router::new()
        .route("/login", post(login_handler));

    let app = Router::new()
        .route("/me", get(me_handler))
        .merge(login_route)
        .route("/logout", post(logout_handler))
        .route("/users/list", get(list_users_handler))
        .route("/users/save", post(save_user_handler))
        .route("/users/delete", post(delete_user_handler))
        .route("/users/admin_request_password_reset", post(admin_request_password_reset_handler))
        .route("/users/user_request_password_reset", post(user_request_password_reset_handler))
        .route("/users/check_reset_token", post(check_reset_token_handler))
        .route("/users/reset_password", post(reset_password_handler))
        .route("/users/change_password", post(change_password_handler))
        .layer(from_fn(options_bypass))
        .layer(cors)
        .layer(CookieManagerLayer::new())
        .with_state(Arc::new(state));

    let url = &env::var(ENV_VARIABLE_NAME_SERVER_URL).unwrap_or_else(|_| "127.0.0.1:3000".into());
    println!("Server starting at {} ...", url);

    let listener = tokio::net::TcpListener::bind(url)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
    
}


// Handler for user login
async fn login_handler(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Json(req): Json<LoginRequest>,
) -> Result<Json<User>, Json<ApiError>> {

    println!("Login attempt for {}", req.email);

    let auth_response = db_sqlite::authenticate_user(
        &state.db,
        &req.email,
        &SecretString::from(req.password))
        .await
        .map_err(|e| { Json(ApiError { message: format!("Database error: {}", e) })
    })?;

    let user = match auth_response {
        Some(r) => r,
        None => return Err(Json(ApiError { message: "Invalid credentials".into() })),
    };

    if !user.is_active {
        return Err(Json(ApiError { message: "Account is inactive".into() }));
    }

    // Create session ID (opaque token)
    let session_id = Uuid::new_v4().to_string();

    db_sqlite::create_user_session(&state.db, &session_id, user.id).await.map_err(|e| {
        Json(ApiError { message: format!("Failed to create session: {}", e) })
    })?;

    cookies.add(
        Cookie::build(("session", session_id))
            .http_only(true)
            .secure(false) // Change to true for production with HTTPS
            .same_site(tower_cookies::cookie::SameSite::Lax)
            .path("/")
            .build(),
    );

    Ok(Json( user ))
}


// Handler for retrieving the currently logged-in user
async fn me_handler(
    auth: AuthUser,
//    State(state): State<Arc<AppState>>,
//    cookies: Cookies,
) -> Result<Json<User>, Json<ApiError>> {
    auth.require_active().map_err(Json)?;
    Ok(Json(auth.user))
}


// Handler for logging out a user
async fn logout_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
) -> Result<Json<bool>, Json<ApiError>> {
    auth.require_active().map_err(Json)?;
    //println!("Cookies: {:?}", cookies);
    //println!("Logging out session: {}", &auth.session_id);    
    db_sqlite::delete_user_session(&state.db, &auth.session_id).await.map_err(|e| {
        Json(ApiError { message: format!("Database error: {}", e) })
    })?;

    cookies.remove(Cookie::from("session"));

    Ok(Json(true))
}


// Handler for listing all users
async fn list_users_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    //cookies: Cookies,
) -> Result<Json<Vec<User>>, Json<ApiError>> {
    auth.require_admin().map_err(Json)?;
    let rows = db_sqlite::get_all_users(&state.db).await.map_err(|e| {
        Json(ApiError { message: format!("Database error: {}", e) })
    })?.unwrap_or_else(Vec::new);

    let users = rows
        .into_iter()
        .map(|r| User {
            id: r.id,
            username: r.username,
            email: r.email,
            role: r.role,
            is_active: r.is_active,
            change_password: r.change_password,
        })
        .collect();

    Ok(Json(users))
}


// Handler for saving a user (add or edit)
async fn save_user_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(user): Json<User>,
    //cookies: Cookies,
) -> Result<Json<bool>, Json<ApiError>> {
    auth.require_admin().map_err(Json)?;
    let _rows = db_sqlite::save_user(&state.db, user).await.map_err(|e| {
        Json(ApiError { message: format!("Database error: {}", e) })
    })?;

    Ok(Json(true))
}


// Handler for deleting a user
async fn delete_user_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(user_id): Json<i32>,
    //cookies: Cookies,
) -> Result<Json<bool>, Json<ApiError>> {
    auth.require_admin().map_err(Json)?;
    // User's can't delete themselves
    // TODO: Prevent the last admin from being deleted
    if auth.user.id != user_id {
        let _rows = db_sqlite::delete_user(&state.db, user_id).await.map_err(|e| {
            Json(ApiError { message: format!("Database error: {}", e) })
        })?;
    }

    Ok(Json(true))
}


// Handler for an admin requesting a password reset for a user
async fn admin_request_password_reset_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(user): Json<User>,
    //cookies: Cookies,
) -> Result<Json<bool>, (http::StatusCode, Json<ApiError>)> {
    auth.require_admin().map_err(|e| (http::StatusCode::UNAUTHORIZED, Json(e)))?;
    if auth.user.id != user.id { // Prevent user's resetting their own password
   
        let token = security::generate_reset_token();
        let url = &env::var(ENV_VARIABLE_NAME_CLIENT_URL).unwrap_or_else(|_| "127.0.0.1:8080".into());
        let link = format!("{url}/user/reset_password/{token}");
        let body = format!(
            "Click the link below to reset your password:\n\n{link}\n\nThis link expires in 15 minutes.\nIf you did not request a password reset, please contact support."
        );

        let _rows = db_sqlite::insert_reset_token(&state.db, &user.email, &token)
            .await
            .map_err(|e| {
            (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError { message: format!("Database error: {}", e) }),
            )
        })?;

        // TODO: Email the password reset link to the user
        println!("Reset link: {:?}", &body)  // For now print the link in the server console

    }

    Ok(Json(true))
}


// Handler for a user requesting a password reset
async fn user_request_password_reset_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgotPasswordRequest>,
    //cookies: Cookies,
) -> Result<Json<bool>, (http::StatusCode, Json<ApiError>)> {

   
    let token = security::generate_reset_token();
    let url = &env::var(ENV_VARIABLE_NAME_CLIENT_URL).unwrap_or_else(|_| "127.0.0.1:8080".into());
    let link = format!("{url}/user/reset_password/{token}");
    let body = format!(
        "Click the link below to reset your password:\n\n{link}\n\nThis link expires in 15 minutes.\nIf you did not request a password reset, please contact support."
    );

    let _rows = db_sqlite::insert_reset_token(&state.db, &req.email, &token)
        .await
        .map_err(|e| {
        (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { message: format!("Database error: {}", e) }),
        )
    })?;

    // TODO: Email the password reset link to the user
    println!("Reset link: {:?}", &body);  // For now print the link in the server console

    

    Ok(Json(true))
}


// Check if the reset token is valid
async fn check_reset_token_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TokenCheckRequest>,
) -> Result<Json<User>, Json<ApiError>> {

    let token = req.token;
    println!("Token: {}", &token);

    let auth_response = db_sqlite::get_user_by_token(
        &state.db,
        &token)
        .await
        .map_err(|e| { Json(ApiError { message: format!("Database error: {}", e) })
    })?;

    let user = match auth_response {
        Some(r) => r,
        None => return Err(Json(ApiError { message: "Invalid token or inactive user".into() })),
    };
    Ok(Json( user ))
}


// Handler for resetting a user's password
async fn reset_password_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PasswordResetRequest>,
) -> Result<Json<bool>, (http::StatusCode, Json<ApiError>)> {

    if req.token.is_empty() {
        return Err((http::StatusCode::FORBIDDEN, Json(ApiError { message: "Valid token required".to_string() })))
    }

    let _rows = db_sqlite::reset_password(&state.db, &req)
    .await
    .map_err(|e| {
    (
        http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { message: format!("Database error: {}", e) }),
    )
    })?;

    Ok(Json(true))
}


// Handler for resetting a user's password
async fn change_password_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChangePasswordRequest>,
    //cookies: Cookies,
) -> Result<Json<bool>, (http::StatusCode, Json<ApiError>)> {
    auth.require_active().map_err(|e| (http::StatusCode::UNAUTHORIZED, Json(e)))?;

    security::validate_password_strength(&req.new_password)
        .await
        .map_err(|e| (http::StatusCode::BAD_REQUEST, Json(e)))?;

    let _rows = db_sqlite::change_password(
        &state.db,
        req.user.id, 
        &SecretString::from(req.old_password),
        &SecretString::from(req.new_password)
    )
    .await
    .map_err(|e| {
        (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError { message: format!("Database error: {}", e) }),
        )
    })?;

    Ok(Json(true))
}