use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use super::dto::{
    ForgotPasswordDto, LoginDto, LogoutDto, MessageResponseDto, RefreshTokenDto, RegisterDto,
    RegisterOtpDto, ResetPasswordDto, VerifyTokenDto, VerifyTokenResponseDto,
};
use super::service::AuthService;
use super::validation::Validator;
use crate::common::AppError;
use crate::modules::tokens::AuthTokens;

/// Requests an email verification OTP code prior to registration.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register/otp",
    request_body = RegisterOtpDto,
    responses(
        (status = 200, description = "Verification OTP sent", body = MessageResponseDto),
        (status = 400, description = "Validation failed"),
        (status = 409, description = "Email is already registered")
    ),
    tag = "Auth"
)]
pub async fn register_otp(
    State(service): State<Arc<AuthService>>,
    Json(dto): Json<RegisterOtpDto>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    Validator::builder().email(&dto.email).validate()?;

    service.request_register_otp(&dto.email).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Verification OTP sent" })),
    ))
}

/// Completes account registration and returns authentication tokens.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterDto,
    responses(
        (status = 201, description = "Registration successful", body = AuthTokens),
        (status = 400, description = "Validation failed or expired OTP")
    ),
    tag = "Auth"
)]
pub async fn register(
    State(service): State<Arc<AuthService>>,
    Json(dto): Json<RegisterDto>,
) -> Result<(StatusCode, Json<AuthTokens>), AppError> {
    Validator::builder()
        .email(&dto.email)
        .password(&dto.password)
        .otp(&dto.otp)
        .validate()?;

    let tokens = service
        .register(&dto.email, &dto.password, &dto.otp)
        .await?;
    Ok((StatusCode::CREATED, Json(tokens)))
}

/// Authenticates a user with email credentials or a Google ID token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Authentication successful", body = AuthTokens),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Invalid email or password")
    ),
    tag = "Auth"
)]
pub async fn login(
    State(service): State<Arc<AuthService>>,
    Json(dto): Json<LoginDto>,
) -> Result<(StatusCode, Json<AuthTokens>), AppError> {
    match &dto {
        LoginDto::Email { email, password } => {
            Validator::builder()
                .email(email)
                .required("password", password)
                .validate()?;
        }
        LoginDto::Google { id_token } => {
            Validator::builder()
                .required("id_token", id_token)
                .validate()?;
        }
    }

    let tokens = service.login(dto).await?;
    Ok((StatusCode::OK, Json(tokens)))
}

/// Rotates an active refresh token and returns a new token pair.
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenDto,
    responses(
        (status = 200, description = "Token rotation successful", body = AuthTokens),
        (status = 401, description = "Invalid or expired refresh token")
    ),
    tag = "Auth"
)]
pub async fn refresh(
    State(service): State<Arc<AuthService>>,
    Json(dto): Json<RefreshTokenDto>,
) -> Result<(StatusCode, Json<AuthTokens>), AppError> {
    let tokens = service.refresh(&dto.refresh_token).await?;
    Ok((StatusCode::OK, Json(tokens)))
}

/// Terminates an active user session and hard-deletes the refresh token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = LogoutDto,
    responses(
        (status = 200, description = "Logged out successfully", body = MessageResponseDto)
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(service): State<Arc<AuthService>>,
    Json(dto): Json<LogoutDto>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    service
        .logout(&dto.access_token, &dto.refresh_token)
        .await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Logged out successfully" })),
    ))
}

/// Requests a temporary password reset code.
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    request_body = ForgotPasswordDto,
    responses(
        (status = 200, description = "Reset code sent if email registered", body = MessageResponseDto),
        (status = 400, description = "Validation failed")
    ),
    tag = "Auth"
)]
pub async fn forgot_password(
    State(service): State<Arc<AuthService>>,
    Json(dto): Json<ForgotPasswordDto>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    Validator::builder().email(&dto.email).validate()?;

    service.request_password_reset_otp(&dto.email).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "If this email is registered, a reset code was sent" })),
    ))
}

/// Verifies a password reset OTP and updates the account password.
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body = ResetPasswordDto,
    responses(
        (status = 200, description = "Password updated successfully", body = MessageResponseDto),
        (status = 400, description = "Validation failed or expired OTP")
    ),
    tag = "Auth"
)]
pub async fn reset_password(
    State(service): State<Arc<AuthService>>,
    Json(dto): Json<ResetPasswordDto>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    Validator::builder()
        .email(&dto.email)
        .otp(&dto.otp)
        .new_password(&dto.new_password)
        .validate()?;

    service
        .reset_password(&dto.email, &dto.otp, &dto.new_password)
        .await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Password updated successfully" })),
    ))
}

/// Soft-deletes user account, anonymizes personal data, and revokes active sessions.
#[utoipa::path(
    delete,
    path = "/api/v1/auth/account",
    responses(
        (status = 200, description = "Account deleted successfully", body = MessageResponseDto),
        (status = 401, description = "Missing or invalid authorization header"),
        (status = 403, description = "Banned account cannot be deleted")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Auth"
)]
pub async fn delete_account(
    State(service): State<Arc<AuthService>>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Unauthorized("Invalid Bearer token".into()))?;

    service.delete_account(token).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Account deleted successfully" })),
    ))
}

/// Internal token verification endpoint used by gateways and microservices.
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify",
    request_body = VerifyTokenDto,
    responses(
        (status = 200, description = "Token is valid", body = VerifyTokenResponseDto),
        (status = 401, description = "Invalid or expired access token")
    ),
    tag = "Auth"
)]
pub async fn verify_token(
    State(service): State<Arc<AuthService>>,
    Json(payload): Json<VerifyTokenDto>,
) -> Result<(StatusCode, Json<VerifyTokenResponseDto>), AppError> {
    let session = service.verify_token(&payload.token).await?;
    Ok((
        StatusCode::OK,
        Json(VerifyTokenResponseDto {
            user_id: session.user_id,
            email: session.email,
        }),
    ))
}

/// Health check and readiness probe for container orchestrators.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy")
    ),
    tag = "Health"
)]
pub async fn health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}
