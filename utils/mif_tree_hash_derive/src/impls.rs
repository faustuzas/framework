use super::*;

pub fn tree_hash_derive(item_ast: &syn::DeriveInput) -> TokenStream {
    let name = &item_ast.ident;
    let (impl_generics, type_generics, where_clause) = &item_ast.generics.split_for_impl();

    let struct_meta = match &item.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Tree hash derive supports only structs."),
    };

    let mut append_leaves = vec![];
    struct_meta.fields.iter()
        .filter(|field| should_hash_field(*field))
        .for_each(|field| {
            let field_name = extract_ident(field);

            append_leaves.push(quote! {
                leaves.append(&mut self.#idents.tree_hash_root())
            });
        });

    let generated = quote! {
        impl #impl_generics tree_hash::TreeHash for #name #ty_generics #where_clause {
            fn tree_hash_type() -> tree_hash::TreeHashType {
                tree_hash::TreeHashType::Container
            }

            fn tree_hash_packed_encoding(&self) -> Vec<u8> {
                unreachable!("Struct should not be packed.")
            }

            fn tree_hash_packing_factor() -> usize {
                unreachable!("Struct should not be packed.")
            }

            fn tree_hash_root(&self) -> Vec<u8> {
                let mut leaves = Vec::with_capacity(4 * tree_hash::HASH_SIZE);

                #(
                    append_leaves;
                )*

                tree_hash::merkle_root(&leaves, 0)
            }
        }
    };

    generated.into()
}

pub fn tree_hash_signed_root_derive(item_ast: &syn::DeriveInput) -> TokenStream {
    let name = &item_ast.ident;
    let (impl_generics, type_generics, where_clause) = &item_ast.generics.split_for_impl();

    let struct_meta = match &item.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Tree hash derive supports only structs."),
    };

    let mut append_leaves = vec![];
    struct_meta.fields.iter()
        .filter(|field| should_use_field_for_signed_root(*field))
        .for_each(|field| {
            let field_name = extract_ident(field);

            append_leaves.push(quote! {
                leaves.append(&mut self.#idents.tree_hash_root())
            });
        });
    let leaves_count = append_leaves.len();

    let generated = quote! {
        impl #impl_generics tree_hash::SignedRoot for #name #ty_generics #where_clause {
            fn signed_root(&self) -> Vec<u8> {
                let mut leaves = Vec::with_capacity(#leaves_count * tree_hash::HASH_SIZE);

                #(
                    append_leaves;
                )*

                tree_hash::merkle_root(&leaves, 0)
            }
        }
    };

    generated.into()
}

fn should_use_field_for_signed_root(field: &syn::Field) -> bool {
    !field.attrs.iter()
        .any(|attr|
            attr.path.is_ident("tree_hash")
                && attr.tts.to_string().replace(" ", "") == "(skip_hashing)")
}

fn should_hash_field(field: &syn::Field) -> bool {
    !field.attrs.iter()
        .any(|attr|
            attr.path.is_ident("tree_hash")
                && attr.tts.to_string().replace(" ", "") == "(skip_hashing)")
}

fn extract_ident(field: &syn::Field) -> &syn::Ident {
    match &field.ident {
        Some(ident) => ident,
        _ => panic!("Hashing supports only named fields")
    }
}