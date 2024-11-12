use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStatic};

#[proc_macro_attribute]
pub fn expose_lint_info(_: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the static item
    let input = parse_macro_input!(item as ItemStatic);
    let ident = &input.ident;

    // Generate the FFI function using the static's identifier
    let expanded = quote! {
        #input

        use common::{CLintInfo, LintInfo};

        #[no_mangle]
        pub extern "C" fn lint_info() -> *mut CLintInfo {
            LintInfo::create_lint_info(&#ident)
        }
    };

    TokenStream::from(expanded)
}
