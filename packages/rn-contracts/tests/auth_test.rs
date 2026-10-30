#[cfg(feature = "auth")]
mod tests {
    use rn_contracts::auth::{VerifyTokenRequest, VerifyTokenResponse};
    use uuid::Uuid;

    #[test]
    fn test_verify_token_serialization() {
        let req = VerifyTokenRequest {
            token: "at_test_token_123".into(),
        };
        let req_json = serde_json::to_string(&req).unwrap();
        assert!(req_json.contains("at_test_token_123"));

        let user_id = Uuid::now_v7();
        let res = VerifyTokenResponse {
            user_id,
            email: "user@example.com".into(),
        };
        let res_json = serde_json::to_string(&res).unwrap();
        assert!(res_json.contains("user@example.com"));

        let deserialized: VerifyTokenResponse = serde_json::from_str(&res_json).unwrap();
        assert_eq!(deserialized, res);
    }
}
