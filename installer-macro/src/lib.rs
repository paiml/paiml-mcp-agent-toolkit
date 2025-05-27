use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, ReturnType};

mod mir_lowering;
mod shell_ast;
mod shell_emitter;
mod verification;

use crate::verification::{verify_determinism, verify_posix_compliance};

#[proc_macro_attribute]
pub fn shell_installer(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);

    // Extract function metadata
    let fn_name = &func.sig.ident;
    let const_name = format_ident!("{}_SHELL", fn_name.to_string().to_uppercase());

    // Parse and verify function signature
    match &func.sig.output {
        ReturnType::Type(_, ty) => {
            let ty_str = quote!(#ty).to_string();
            if !ty_str.contains("Result") {
                panic!("Shell installer must return Result<(), Error>");
            }
        }
        _ => panic!("Shell installer must have explicit return type"),
    }

    // For proof of concept, generate a static shell script
    // In a real implementation, this would analyze the function AST
    let shell_script = include_str!("../../scripts/install.sh");

    // Verify at compile time (only if shellcheck is available)
    if std::process::Command::new("which")
        .arg("shellcheck")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        match verify_posix_compliance(shell_script) {
            Ok(_) => {}
            Err(e) => panic!("Generated shell is not POSIX compliant: {}", e),
        }
    }

    match verify_determinism(shell_script) {
        Ok(_) => {}
        Err(e) => panic!("Shell generation is non-deterministic: {}", e),
    }

    // Emit both Rust function and shell constant
    let output = quote! {
        #func

        #[doc = "Generated POSIX shell installer script"]
        pub const #const_name: &str = #shell_script;
    };

    output.into()
}
