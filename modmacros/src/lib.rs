//! Bevy Mod Macros
//!
//! This crate provides procedural macros for the Bevy modding framework.
//! These macros make it easier to define mods, systems, components, and resources.
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, Token, parse::Parse, parse::ParseStream, parse_macro_input,
    punctuated::Punctuated,
};

/// System macro.
///
/// Add this macro to your Fn to insert to your system
#[proc_macro_attribute]
pub fn system(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(input as ItemFn);

    // Get the function name
    let fn_name = &input_fn.sig.ident;

    // Generate the export function name
    let export_fn_name = quote::format_ident!("__mod_export_system_{}", fn_name);
    // Generate the get info function name
    let info_fn_name = quote::format_ident!("__mod_info_system_{}", fn_name);

    // Convert function name to string for SystemInfo
    let fn_name_str = export_fn_name.to_string();

    // Generate the output tokens
    let expanded = quote! {
        // Keep the original function
        #input_fn

        // Generate the export function
        #[unsafe(no_mangle)]
        pub extern "C" fn #export_fn_name() {
            // Call the original function
            #fn_name();
        }

        // Get system info
        #[unsafe(no_mangle)]
        pub extern "C" fn #info_fn_name() -> *const SystemInfo {
            // Use a static variable to store the SystemInfo to ensure it lives long enough
            static mut SYSTEM_INFO: Option<SystemInfo> = None;
            static INIT: std::sync::Once = std::sync::Once::new();

            unsafe {
                INIT.call_once(|| {
                    let mut info = SystemInfo {
                        export_name: [0; 64],
                    };

                    // Copy the function name into the array, ensuring it's null-terminated
                    let name_bytes = #fn_name_str.as_bytes();
                    let len = name_bytes.len().min(63); // Leave space for null terminator
                    info.export_name[..len].copy_from_slice(&name_bytes[..len]);
                    info.export_name[len] = 0; // Null terminator

                    SYSTEM_INFO = Some(info);
                });

                SYSTEM_INFO.as_ref().unwrap() as *const SystemInfo
            }
        }
    };

    TokenStream::from(expanded)
}

/// Arguments for the mod macro
struct ModArgs {
    systems: Punctuated<Ident, Token![,]>,
}

impl Parse for ModArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let systems = Punctuated::parse_separated_nonempty(input)?;
        Ok(ModArgs { systems })
    }
}

/// Mod macro.
///
/// This macro is used to define a mod and list all its systems.
/// It generates a function that returns information about all systems in the mod.
#[proc_macro]
pub fn mod_def(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as ModArgs);

    // Create a vector of system names
    let system_names: Vec<_> = args
        .systems
        .iter()
        .map(|name| {
            let name_str = name.to_string();
            quote! {
                (#name_str)
            }
        })
        .collect();

    let systems_count = args.systems.len();

    let expanded = quote! {
        // Generate a static array with system name
        #[unsafe(no_mangle)]
        pub static MOD_SYSTEM_NAMES: [&'static str; #systems_count] = [
            #(#system_names),*
        ];

        // Generate a function that returns the count of systems
        #[unsafe(no_mangle)]
        pub extern "C" fn __mod_get_systems_count() -> usize {
            #systems_count
        }

        // Get ptr of system names
        #[unsafe(no_mangle)]
        pub extern "C" fn __mod_get_systems_names_ptr() -> *const u8 {
            MOD_SYSTEM_NAMES.as_ptr() as *const u8
        }

        // Generate a function that returns the mod name (derived from the crate name)
        #[unsafe(no_mangle)]
        pub extern "C" fn __mod_get_name() -> *const u8 {
            // Get the crate name from environment variable
            let crate_name = env!("CARGO_PKG_NAME");
            // We need to store the string in a static variable to return a valid pointer
            static mut MOD_NAME: Option<String> = None;
            static INIT: std::sync::Once = std::sync::Once::new();

            unsafe {
                INIT.call_once(|| {
                    MOD_NAME = Some(format!("{}\0", crate_name));
                });
                MOD_NAME.as_ref().unwrap().as_ptr()
            }
        }
    };

    TokenStream::from(expanded)
}
