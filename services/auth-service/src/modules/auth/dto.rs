use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Request DTO for requesting email registration verification OTP.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterOtpDto {
    pub email: String,
}

/// Request DTO for completing user registration.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterDto {
    pub email: String,
    pub password: String,
    pub otp: String,
}

/// Request DTO for user login supporting local credentials and Google ID tokens.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LoginDto {
    Email { email: String, password: String },
    Google { id_token: String },
}

/// Request DTO for rotating an active refresh token.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenDto {
    pub refresh_token: String,
}

/// Request DTO for terminating active session.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LogoutDto {
    pub access_token: String,
    pub refresh_token: String,
}

/// Request DTO for requesting password reset OTP.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ForgotPasswordDto {
    pub email: String,
}

/// Request DTO for completing password reset.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordDto {
    pub email: String,
    pub otp: String,
    pub new_password: String,
}

/// Request DTO for internal token verification.
#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyTokenDto {
    pub token: String,
}

/// Response DTO containing verified user session info.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyTokenResponseDto {
    pub user_id: Uuid,
    pub email: String,
}

/// Generic message response payload.
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponseDto {
    pub message: String,
}
