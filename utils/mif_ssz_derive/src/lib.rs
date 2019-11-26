#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod impls;

#[proc_macro_derive(Encode, attributes(ssz))]
pub fn ssz_encode_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impls::ssz_encode_derive(&ast)
}

#[proc_macro_derive(Decode, attributes(ssz))]
pub fn ssz_decode_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impls::ssz_decode_derive(&ast)
}