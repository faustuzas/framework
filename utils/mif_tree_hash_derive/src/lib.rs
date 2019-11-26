#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod impls;

#[proc_macro_derive(TreeHash, attributes(tree_hash))]
pub fn tree_hash_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impls::tree_hash_derive(&ast)
}

#[proc_macro_derive(SignedRoot, attributes(signed_root))]
pub fn tree_hash_signed_root_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    impls::tree_hash_signed_root_derive(&ast)
}