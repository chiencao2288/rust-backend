use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::common::AppError;

/// Simple zero-dependency template engine replacing `{{key}}` placeholders with parameter values.
pub struct TemplateEngine;

impl TemplateEngine {
    /// Renders an HTML template with the given dynamic parameter map.
    ///
    /// Looks up `templates/{template_name}.html` relative to working directory or fallback locations.
    /// Replaces every occurrence of `{{key}}` with its corresponding value.
    pub fn render(
        template_name: &str,
        params: &HashMap<String, String>,
    ) -> Result<String, AppError> {
        let content = load_template_content(template_name)?;

        let mut output = content;
        for (key, val) in params {
            let placeholder = format!("{{{{{key}}}}}");
            output = output.replace(&placeholder, val);
        }

        // Apply default fallback for {{app_name}} if not explicitly provided
        if !params.contains_key("app_name") {
            output = output.replace("{{app_name}}", "RN Auth Platform");
        }

        Ok(output)
    }
}

fn load_template_content(name: &str) -> Result<String, AppError> {
    let candidate_paths = [
        format!("templates/{name}.html"),
        format!("services/noti-service/templates/{name}.html"),
    ];

    for path_str in &candidate_paths {
        let path = Path::new(path_str);
        if path.exists() {
            return fs::read_to_string(path)
                .map_err(|e| AppError::Internal(format!("Failed to read template `{name}`: {e}")));
        }
    }

    // Default minimal HTML wrapper fallback if file not on disk
    Ok(format!(
        "<!DOCTYPE html><html><body><h2>RN Notification</h2><p>Template: {name}</p></body></html>"
    ))
}
