use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;

use crate::common::AppError;

/// Field-level validation failure details.
#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub code: &'static str,
    pub message: &'static str,
}

/// Fluent validation builder for validating request payloads in controller handlers.
#[derive(Default, Debug)]
pub struct Validator {
    errors: BTreeMap<&'static str, FieldError>,
}

impl Validator {
    /// Creates a new fluent validation builder.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Validates an email address.
    pub fn email(mut self, email: &str) -> Self {
        let trimmed = email.trim();
        if trimmed.is_empty() || !trimmed.contains('@') || !trimmed.contains('.') {
            self.errors.insert(
                "email",
                FieldError {
                    code: "INVALID_EMAIL",
                    message: "email must be a valid email address",
                },
            );
        }
        self
    }

    /// Validates password complexity:
    /// - Minimum 8 characters
    /// - At least one uppercase letter
    /// - At least one number
    /// - At least one special character
    pub fn password(mut self, password: &str) -> Self {
        if password.len() < 8 {
            self.errors.insert(
                "password",
                FieldError {
                    code: "PASSWORD_TOO_SHORT",
                    message: "password must be at least 8 characters long",
                },
            );
            return self;
        }

        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if !has_uppercase || !has_digit || !has_special {
            self.errors.insert(
                "password",
                FieldError {
                    code: "PASSWORD_INVALID_FORMAT",
                    message: "password must contain at least one uppercase letter, one number, and one special character",
                },
            );
        }
        self
    }

    /// Validates new password complexity on reset password requests.
    pub fn new_password(mut self, password: &str) -> Self {
        if password.len() < 8 {
            self.errors.insert(
                "password",
                FieldError {
                    code: "PASSWORD_TOO_SHORT",
                    message: "password must be at least 8 characters long",
                },
            );
            return self;
        }

        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if !has_uppercase || !has_digit || !has_special {
            self.errors.insert(
                "password",
                FieldError {
                    code: "PASSWORD_INVALID_FORMAT",
                    message: "password must contain at least one uppercase letter, one number, and one special character",
                },
            );
        }
        self
    }

    /// Validates a 6-digit numeric OTP code.
    pub fn otp(mut self, otp: &str) -> Self {
        let trimmed = otp.trim();
        if trimmed.len() != 6 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
            self.errors.insert(
                "otp",
                FieldError {
                    code: "INVALID_OTP_FORMAT",
                    message: "otp must be a 6-digit numeric code",
                },
            );
        }
        self
    }

    /// Validates that a string field is not empty.
    pub fn required(mut self, field: &'static str, value: &str) -> Self {
        if value.trim().is_empty() {
            self.errors.insert(
                field,
                FieldError {
                    code: "REQUIRED_FIELD",
                    message: "this field is required",
                },
            );
        }
        self
    }

    /// Executes validation and returns `Err(AppError::ValidationFailed)` if any errors were accumulated.
    pub fn validate(self) -> Result<(), AppError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            let details = serde_json::to_value(&self.errors).unwrap_or_else(|_| json!({}));
            Err(AppError::ValidationFailed {
                message: "Validation failed".into(),
                details,
            })
        }
    }
}
