use noti_service::modules::email::TemplateEngine;
use std::collections::HashMap;

#[test]
fn test_register_otp_template_rendering() {
    let mut params = HashMap::new();
    params.insert("otp".into(), "654321".into());
    params.insert("ttl_min".into(), "5".into());
    params.insert("app_name".into(), "My Custom App".into());

    let rendered = TemplateEngine::render("register_otp", &params).expect("Template should render");

    assert!(rendered.contains("654321"));
    assert!(rendered.contains("5 minutes"));
    assert!(rendered.contains("My Custom App"));
    assert!(!rendered.contains("{{otp}}"));
    assert!(!rendered.contains("{{ttl_min}}"));
    assert!(!rendered.contains("{{app_name}}"));
}

#[test]
fn test_reset_password_template_rendering() {
    let mut params = HashMap::new();
    params.insert("otp".into(), "112233".into());
    params.insert("ttl_min".into(), "10".into());

    let rendered =
        TemplateEngine::render("reset_password", &params).expect("Template should render");

    assert!(rendered.contains("112233"));
    assert!(rendered.contains("10 minutes"));
    assert!(rendered.contains("RN Auth Platform")); // Default fallback
}

#[test]
fn test_missing_template_fallback() {
    let params = HashMap::new();
    let rendered =
        TemplateEngine::render("non_existent_template", &params).expect("Should fallback cleanly");

    assert!(rendered.contains("non_existent_template"));
}
