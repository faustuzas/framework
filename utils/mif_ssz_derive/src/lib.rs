#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

mod impls;
use impls::*;

#[proc_macro_derive(Encode, attributes(ssz))]
pub fn ssz_encode_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_ssz_encode_derive(&ast)
}

#[proc_macro_derive(Decode, attributes(ssz))]
pub fn ssz_decode_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_ssz_decode_derive(&ast)
}