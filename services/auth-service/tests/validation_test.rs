use auth_service::common::AppError;
use auth_service::modules::auth::validation::Validator;

#[test]
fn test_fluent_validator_builder_fails_with_exact_field_details() {
    let result = Validator::builder()
        .email("invalid-email")
        .password("simple")
        .otp("12")
        .validate();

    assert!(result.is_err());

    match result.err().unwrap() {
        AppError::ValidationFailed { message, details } => {
            assert_eq!(message, "Validation failed");
            assert_eq!(details["email"]["code"], "INVALID_EMAIL");
            assert_eq!(
                details["email"]["message"],
                "email must be a valid email address"
            );
            assert_eq!(details["password"]["code"], "PASSWORD_TOO_SHORT");
            assert_eq!(details["otp"]["code"], "INVALID_OTP_FORMAT");
        }
        other => panic!("Expected ValidationFailed, got: {:?}", other),
    }
}

#[test]
fn test_fluent_validator_builder_password_complexity_rules() {
    let result = Validator::builder()
        .email("test@example.com")
        .password("alllowercase123!")
        .otp("123456")
        .validate();

    assert!(result.is_err());

    match result.err().unwrap() {
        AppError::ValidationFailed { details, .. } => {
            assert_eq!(details["password"]["code"], "PASSWORD_INVALID_FORMAT");
        }
        other => panic!("Expected ValidationFailed, got: {:?}", other),
    }
}

#[test]
fn test_fluent_validator_builder_passes_for_valid_inputs() {
    let result = Validator::builder()
        .email("user@example.com")
        .password("StrongPassword123!")
        .otp("839201")
        .validate();

    assert!(result.is_ok());
}
