//! Sdk macros.
//!
//! Use these macros in sdk to define component, resource, event

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, Lit, MetaNameValue};

/// Parse the id from the macro arguments
fn parse_id_from_args(args: proc_macro::TokenStream) -> String {
    let args_str = args.to_string();
    if args_str.is_empty() {
        return String::new();
    }

    // Parse as meta attributes
    let args_parsed = syn::parse::<Meta>(args).expect("Failed to parse args");
    
    match args_parsed {
        Meta::NameValue(MetaNameValue { path, value, .. }) => {
            if path.is_ident("id") {
                match value {
                    syn::Expr::Lit(lit) => {
                        match lit.lit {
                            Lit::Str(s) => s.value(),
                            _ => panic!("Expected string literal for id"),
                        }
                    }
                    _ => panic!("Expected literal for id"),
                }
            } else {
                panic!("Expected 'id' attribute");
            }
        }
        _ => panic!("Expected named value attribute"),
    }
}

/// Component macro.
///
/// This macro is used to mark a component that can be queried by mods.
/// It will register the component with an ID.
#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let id = parse_id_from_args(args);

    let struct_name = &derive_input.ident;
    let component_id = if id.is_empty() {
        struct_name.to_string()
    } else {
        id
    };

    let expanded = quote! {
        // Original struct
        #derive_input

        // Component ID registration
        impl #struct_name {
            /// Get the component ID
            pub const fn component_id() -> &'static str {
                #component_id
            }
        }
    };

    TokenStream::from(expanded)
}
