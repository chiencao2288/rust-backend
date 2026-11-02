use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Token};

/// Parsed `#[status(STATUS)]` or `#[status(STATUS, code = "CODE")]` attribute.
struct StatusAttr {
    status: syn::Ident,
    code: Option<String>,
}

impl Parse for StatusAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let status: syn::Ident = input.parse()?;
        let mut code = None;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident: syn::Ident = input.parse()?;
            if ident == "code" {
                input.parse::<Token![=]>()?;
                let lit: syn::LitStr = input.parse()?;
                code = Some(lit.value());
            } else {
                return Err(syn::Error::new(ident.span(), "expected `code`"));
            }
        }

        Ok(StatusAttr { status, code })
    }
}

pub fn derive_api_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => panic!("ApiError can only be derived for enums"),
    };

    let mut code_arms = Vec::new();
    let mut status_arms = Vec::new();
    let mut into_response_arms = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;

        let status_attr = variant
            .attrs
            .iter()
            .find(|a| a.path().is_ident("status"))
            .unwrap_or_else(|| {
                panic!("Variant `{variant_name}` is missing #[status(STATUS_CODE)] attribute")
            });

        let parsed: StatusAttr = status_attr.parse_args().unwrap_or_else(|e| {
            panic!("Variant `{variant_name}`: invalid #[status] attribute: {e}")
        });

        let status_ident = &parsed.status;
        let code_str = parsed
            .code
            .clone()
            .unwrap_or_else(|| to_screaming_snake_case(&variant_name.to_string()));

        let is_internal = status_ident == "INTERNAL_SERVER_ERROR";

        // code() arm
        code_arms.push(quote! {
            Self::#variant_name { .. } => #code_str,
        });

        // status_code() arm
        status_arms.push(quote! {
            Self::#variant_name { .. } => axum::http::StatusCode::#status_ident,
        });

        // into_response() arm
        let arm = match &variant.fields {
            Fields::Named(fields) => {
                let has_message = fields
                    .named
                    .iter()
                    .any(|f| f.ident.as_ref().is_some_and(|id| id == "message"));
                let has_details = fields
                    .named
                    .iter()
                    .any(|f| f.ident.as_ref().is_some_and(|id| id == "details"));

                if has_message && has_details {
                    quote! {
                        Self::#variant_name { message, details, .. } => {
                            let status = axum::http::StatusCode::#status_ident;
                            let code = #code_str.to_string();
                            let message = message.clone();
                            let details: Option<serde_json::Value> = Some(details.clone());
                            (status, code, message, details)
                        }
                    }
                } else if has_message {
                    quote! {
                        Self::#variant_name { message, .. } => {
                            let status = axum::http::StatusCode::#status_ident;
                            let code = #code_str.to_string();
                            let message = message.clone();
                            let details: Option<serde_json::Value> = None;
                            (status, code, message, details)
                        }
                    }
                } else {
                    quote! {
                        Self::#variant_name { .. } => {
                            let status = axum::http::StatusCode::#status_ident;
                            let code = #code_str.to_string();
                            let message = self.to_string();
                            let details: Option<serde_json::Value> = None;
                            (status, code, message, details)
                        }
                    }
                }
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                if is_internal {
                    quote! {
                        Self::#variant_name(msg) => {
                            tracing::error!("Internal server error: {msg}");
                            let status = axum::http::StatusCode::#status_ident;
                            let code = #code_str.to_string();
                            let message = "Internal server error".to_string();
                            let details: Option<serde_json::Value> = None;
                            (status, code, message, details)
                        }
                    }
                } else {
                    quote! {
                        Self::#variant_name(msg) => {
                            let status = axum::http::StatusCode::#status_ident;
                            let code = #code_str.to_string();
                            let message = msg.clone();
                            let details: Option<serde_json::Value> = None;
                            (status, code, message, details)
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => {
                        let status = axum::http::StatusCode::#status_ident;
                        let code = #code_str.to_string();
                        let message = self.to_string();
                        let details: Option<serde_json::Value> = None;
                        (status, code, message, details)
                    }
                }
            }
            _ => panic!(
                "ApiError: unsupported field layout for variant `{}`",
                variant_name
            ),
        };

        into_response_arms.push(arm);
    }

    quote! {
        impl #name {
            /// Returns a machine-readable, stable error code for client-side condition handling.
            pub fn code(&self) -> &'static str {
                match self {
                    #(#code_arms)*
                }
            }

            /// Returns the HTTP status code associated with this error.
            pub fn status_code(&self) -> axum::http::StatusCode {
                match self {
                    #(#status_arms)*
                }
            }
        }

        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                let (status, code, message, details) = match &self {
                    #(#into_response_arms)*
                };

                let mut data = serde_json::json!({
                    "code": code,
                    "message": message,
                });

                if let Some(d) = details {
                    if let Some(obj) = data.as_object_mut() {
                        obj.insert("details".to_string(), d);
                    }
                }

                let body = serde_json::json!({
                    "success": false,
                    "data": data,
                });

                axum::response::IntoResponse::into_response((status, axum::Json(body)))
            }
        }
    }
    .into()
}

fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        let c = chars[i];
        if c.is_uppercase() && i > 0 {
            let prev_lower = chars[i - 1].is_lowercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            if prev_lower || next_lower {
                result.push('_');
            }
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}
