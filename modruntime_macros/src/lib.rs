//! Runtime macros
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// This macro is used to mark a component that can be accessed by mods.
/// It will automatically implement serde serialization/deserialization and register the component.
#[proc_macro_attribute]
pub fn mod_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let id = parse_id_from_args(args);

    let struct_name = &derive_input.ident;
    let component_id = if id.is_empty() {
        struct_name.to_string()
    } else {
        id
    };

    // Generate a unique static variable name using a hash of the struct name
    let static_name = format!(
        "__MOD_COMPONENT_REGISTRATION_{}",
        struct_name.to_string().to_uppercase()
    );
    let static_ident = syn::Ident::new(&static_name, struct_name.span());

    let expanded = quote! {
        // Original struct with serde and bincode derives
        #[derive(serde::Serialize, serde::Deserialize)]
        #[derive(bevy::prelude::Reflect)]
        #derive_input

        // Component registration
        #[linkme::distributed_slice(COMPONENT_REGISTRY)]
        #[warn(non_upper_case_globals)]
        static #static_ident: modruntime::component::ComponentRegistration =
            modruntime::component::ComponentRegistration {
                id: #component_id,
                serialize_fn: |component: bevy::ptr::Ptr<'_>| -> Vec<u8> {
                    unsafe {
                        let c = component.deref::<#struct_name>();
                        bincode::serde::encode_to_vec(c, bincode::config::standard()).unwrap_or_else(|_| Vec::new())
                    }
                },
                deserialize_fn: |data: &[u8]| -> Box<dyn std::any::Any> {
                    Box::new(bincode::serde::decode_from_slice::
                        <#struct_name, bincode::config::Configuration>(data, bincode::config::standard())
                        .expect("Failed to deserialize component data").0)
                },
                get_type_id: || -> std::any::TypeId {std::any::TypeId::of::<#struct_name>()},
                reg_fn: |mut registry: &mut bevy::reflect::TypeRegistry| {
                    registry.register::<#struct_name>()
                }
            };
    };

    TokenStream::from(expanded)
}

/// This macro is used to mark a resource that can be accessed by mods.
/// It will automatically implement serde serialization/deserialization and register the resource.
#[proc_macro_attribute]
pub fn mod_resource(args: TokenStream, input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let id = parse_id_from_args(args);

    let struct_name = &derive_input.ident;
    let resource_id = if id.is_empty() {
        struct_name.to_string()
    } else {
        id
    };

    // Generate a unique static variable name using a hash of the struct name
    let static_name = format!(
        "__MOD_RESOURCE_REGISTRATION_{}",
        struct_name.to_string().to_uppercase()
    );
    let static_ident = syn::Ident::new(&static_name, struct_name.span());

    let expanded = quote! {
        // Original struct with serde and bincode derives
        #[derive(serde::Serialize, serde::Deserialize)]
        #[derive(bevy::prelude::Reflect)]
        #derive_input

        // Resource registration
        #[linkme::distributed_slice(RESOURCE_REGISTRY)]
        #[warn(non_upper_case_globals)]
        static #static_ident: modruntime::resource::ResourceRegistration =
            modruntime::resource::ResourceRegistration {
                id: #resource_id,
                serialize_fn: |resource: bevy::ptr::Ptr<'_>| -> Vec<u8> {
                    unsafe {
                        let r = resource.deref::<#struct_name>();
                        bincode::serde::encode_to_vec(r, bincode::config::standard()).unwrap_or_else(|_| Vec::new())
                    }
                },
                deserialize_fn: |data: &[u8]| -> Box<dyn std::any::Any> {
                    Box::new(bincode::serde::decode_from_slice::
                        <#struct_name, bincode::config::Configuration>(data, bincode::config::standard())
                        .expect("Failed to deserialize resource data").0)
                },
                get_type_id: || -> std::any::TypeId {std::any::TypeId::of::<#struct_name>()},
                reg_fn: |mut registry: &mut bevy::reflect::TypeRegistry| {
                    registry.register::<#struct_name>()
                }
            };
    };

    TokenStream::from(expanded)
}

/// Parse the id from the macro arguments
fn parse_id_from_args(args: TokenStream) -> String {
    let args_str = args.to_string();

    // Simple parsing for id = "value"
    if let Some(id_start) = args_str.find("id = \"") {
        let id_start = id_start + 6; // Length of id = "
        if let Some(id_end) = args_str[id_start..].find("\"") {
            return args_str[id_start..id_start + id_end].to_string();
        }
    }

    // If no id is found, we'll use the struct name as default
    String::new()
}