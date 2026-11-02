use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr, Token};

/// Parsed `#[env("KEY")]` or `#[env("KEY", fallback = "value")]` attribute.
struct EnvAttr {
    key: String,
    fallback: Option<String>,
}

impl Parse for EnvAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: LitStr = input.parse()?;
        let mut fallback = None;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident: syn::Ident = input.parse()?;
            if ident == "fallback" {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                fallback = Some(value.value());
            } else {
                return Err(syn::Error::new(ident.span(), "expected `fallback`"));
            }
        }

        Ok(EnvAttr {
            key: key.value(),
            fallback,
        })
    }
}

pub fn derive_env_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("EnvConfig only supports structs with named fields"),
        },
        _ => panic!("EnvConfig only supports structs"),
    };

    let mut field_loaders = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let env_attr = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident("env"))
            .unwrap_or_else(|| panic!("Field `{field_name}` is missing #[env(\"KEY\")] attribute"));

        let parsed: EnvAttr = env_attr
            .parse_args()
            .unwrap_or_else(|e| panic!("Field `{field_name}`: invalid #[env] attribute: {e}"));

        let env_key = &parsed.key;
        let is_option = is_option_type(field_ty);

        let loader = if is_option {
            let inner_ty = extract_option_inner(field_ty);
            if is_string_type(&inner_ty) {
                quote! {
                    let #field_name = std::env::var(#env_key)
                        .ok()
                        .filter(|s| !s.trim().is_empty());
                }
            } else {
                quote! {
                    let #field_name = match std::env::var(#env_key) {
                        Ok(val) if !val.trim().is_empty() => {
                            match val.parse::<#inner_ty>() {
                                Ok(v) => Some(v),
                                Err(e) => {
                                    errors.push(format!(
                                        "Invalid {} '{}': {}", #env_key, val, e
                                    ));
                                    None
                                }
                            }
                        }
                        _ => None,
                    };
                }
            }
        } else if let Some(fb) = &parsed.fallback {
            if is_string_type_from_ty(field_ty) {
                quote! {
                    let #field_name = std::env::var(#env_key)
                        .unwrap_or_else(|_| #fb.to_string());
                }
            } else {
                quote! {
                    let #field_name = match std::env::var(#env_key) {
                        Ok(val) => {
                            match val.parse::<#field_ty>() {
                                Ok(v) => v,
                                Err(e) => {
                                    errors.push(format!(
                                        "Invalid {} '{}': {}", #env_key, val, e
                                    ));
                                    #fb.parse::<#field_ty>().unwrap()
                                }
                            }
                        }
                        Err(_) => #fb.parse::<#field_ty>().unwrap(),
                    };
                }
            }
        } else if is_string_type_from_ty(field_ty) {
            quote! {
                let #field_name = match std::env::var(#env_key) {
                    Ok(val) if !val.trim().is_empty() => val,
                    _ => {
                        errors.push(format!(
                            "Missing required environment variable: {}", #env_key
                        ));
                        String::new()
                    }
                };
            }
        } else {
            quote! {
                let #field_name = match std::env::var(#env_key) {
                    Ok(val) => {
                        match val.parse::<#field_ty>() {
                            Ok(v) => v,
                            Err(e) => {
                                errors.push(format!(
                                    "Invalid {} '{}': {}", #env_key, val, e
                                ));
                                Default::default()
                            }
                        }
                    }
                    Err(_) => {
                        errors.push(format!(
                            "Missing required environment variable: {}", #env_key
                        ));
                        Default::default()
                    }
                };
            }
        };

        field_loaders.push(loader);
    }

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    quote! {
        impl #name {
            /// Loads configuration from environment variables.
            ///
            /// Panics with a combined error message listing every missing or invalid variable.
            pub fn load() -> Self {
                let mut errors: Vec<String> = Vec::new();

                #(#field_loaders)*

                if !errors.is_empty() {
                    let msg = errors.join("\n  - ");
                    panic!(
                        "\n=======================================================\n \
                         [FATAL CONFIG ERROR] Service startup aborted!\n  - {}\n\
                         =======================================================\n",
                        msg
                    );
                }

                Self {
                    #(#field_names),*
                }
            }
        }

        impl rn_service::EnvConfig for #name {
            fn load() -> Self {
                Self::load()
            }
        }
    }
    .into()
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn extract_option_inner(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                    return inner.clone();
                }
            }
        }
    }
    panic!("Expected Option<T>");
}

fn is_string_type(ty: &syn::Type) -> bool {
    is_string_type_from_ty(ty)
}

fn is_string_type_from_ty(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }
    false
}
