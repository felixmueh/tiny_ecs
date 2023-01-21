use proc_macro;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn component(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl Component for #ident {}
    };
    output.into()
}
