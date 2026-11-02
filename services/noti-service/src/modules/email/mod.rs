#[path = "email.consumer.rs"]
pub mod consumer;
pub mod providers;
#[path = "email.service.rs"]
pub mod service;
#[path = "email.template.rs"]
pub mod template;

pub use consumer::start_email_consumer;
pub use providers::{create_email_provider, EmailProvider};
pub use service::EmailService;
pub use template::TemplateEngine;
