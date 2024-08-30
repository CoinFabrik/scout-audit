use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn scout_allow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
