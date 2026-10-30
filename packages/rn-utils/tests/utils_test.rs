use rn_utils::{
    generate_numeric_otp, generate_secure_token, hash_password, sanitize_json_value, sha256_hash,
    verify_password,
};

#[test]
fn test_password_hashing_and_verification() {
    let password = "my_super_secret_password";
    let hash = hash_password(password).expect("hashing should succeed");

    assert!(verify_password(password, &hash).expect("verification should succeed"));
    assert!(!verify_password("wrong_password", &hash).expect("verification should succeed"));
}

#[test]
fn test_token_generation() {
    let token1 = generate_secure_token(32);
    let token2 = generate_secure_token(32);

    assert_eq!(token1.len(), 64);
    assert_eq!(token2.len(), 64);
    assert_ne!(token1, token2);
}

#[test]
fn test_otp_generation() {
    let otp = generate_numeric_otp(6);
    assert_eq!(otp.len(), 6);
    assert!(otp.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_sha256_hashing() {
    let hash = sha256_hash("hello world");
    assert_eq!(
        hash,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_sanitize_json_value_masks_passwords_and_tokens() {
    let mut payload = serde_json::json!({
        "email": "user@example.com",
        "password": "my_secret_password",
        "nested": {
            "token": "secret_jwt",
            "safe_field": 123
        },
        "items": [
            { "new_password": "new_secret" },
            { "safe": true }
        ]
    });

    sanitize_json_value(&mut payload);

    assert_eq!(payload["password"], "******");
    assert_eq!(payload["nested"]["token"], "******");
    assert_eq!(payload["items"][0]["new_password"], "******");
    assert_eq!(payload["email"], "user@example.com");
    assert_eq!(payload["nested"]["safe_field"], 123);
    assert_eq!(payload["items"][1]["safe"], true);
}
