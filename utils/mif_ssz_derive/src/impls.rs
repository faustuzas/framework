use super::*;

pub fn impl_ssz_encode_derive(item_ast: &syn::DeriveInput) -> TokenStream {
    let name = &item_ast.ident;
    let (impl_generics, type_generics, where_clause) = &item_ast.generics.split_for_impl();

    let struct_meta = match &item_ast.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Encode derivce macro supports only structs")
    };

    // you have to clone it because variable can be used only one time in the pattern
    let field_idents = extract_serializable_idents(struct_meta);
    let field_idents_1 = field_idents.clone();

    let field_types = extract_serializable_types(struct_meta);
    let field_types_1 = field_types.clone();
    let field_types_2 = field_types.clone();
    let field_types_3 = field_types.clone();
    let field_types_4 = field_types.clone();

    let generated = quote! {
        impl #impl_generics Encode for #name #type_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
            #(
                <#field_types as ssz::Encode>::is_ssz_fixed_len() &&
            )*
                true
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                let offset = #(
                        <#field_types_1 as ssz::Encode>::ssz_fixed_len() +
                    )*
                        0;

                let mut encoder = ssz::SszEncoder::container(buf, offset);

                #(
                    encoder.append(&self.#field_idents);
                )*

                encoder.finalize();
            }

            fn ssz_fixed_len() -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    #(
                        <#field_types_2 as ssz::Encode>::ssz_fixed_len() +
                    )*
                        0
                } else {
                    ssz::BYTES_PER_LENGTH_OFFSET
                }
            }

            fn ssz_bytes_len(&self) -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    return <Self as ssz::Encode>::ssz_fixed_len()
                }

                let mut len = 0;

                #(
                    if <#field_types_3 as ssz::Encode>::is_ssz_fixed_len() {
                        len += <#field_types_4 as ssz::Encode>::ssz_fixed_len();
                    } else {
                        len += ssz::BYTES_PER_LENGTH_OFFSET;
                        len += self.#field_idents_1.ssz_bytes_len();
                    }
                )*

                return len
            }
        }
    };

    generated.into()
}

pub fn impl_ssz_decode_derive(item_ast: &syn::DeriveInput) -> TokenStream {
    let name = &item_ast.ident;
    let (impl_generics, type_generics, where_clause) = &item_ast.generics.split_for_impl();

    let struct_meta = match &item_ast.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Encode derivce macro supports only structs")
    };

    // you have to clone it because variable can be used only one time in the pattern
    let field_idents = extract_serializable_idents(struct_meta);

    let field_types = extract_serializable_types(struct_meta);
    let field_types_1 = field_types.clone();
    let field_types_2 = field_types.clone();

    let generated = quote! {
        impl #impl_generics ssz::Decode for #name #type_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                #(
                    <#field_types as ssz::Decode>::is_ssz_fixed_len() &&
                )*
                    true
            }

            fn ssz_fixed_len() -> usize {
                if <Self as ssz::Decode>::is_ssz_fixed_len() {
                    #(
                        <#field_types_1 as ssz::Decode>::ssz_fixed_len() +
                    )*
                        0
                } else {
                    ssz::BYTES_PER_LENGTH_OFFSET
                }
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                let mut builder = ssz::SszDecoderBuilder::new(bytes);

                #(
                    builder.register_type::<#field_types_2>()?;
                )*

                let mut decoder = builder.build()?;

                Ok(Self {
                    #(
                        #field_idents: decoder.decode_next()?,
                    )*
                })
            }
        }
    };

    generated.into()
}

fn extract_serializable_idents(struct_meta: &syn::DataStruct) -> Vec<&syn::Ident> {
    struct_meta.fields.iter()
        .map(|f| match &f.ident {
            Some(ident) => ident,
            _ => panic!("ssz_derive only supports named fields")
        })
        .collect()
}

fn extract_serializable_types(struct_meta: &syn::DataStruct) -> Vec<&syn::Type> {
    struct_meta.fields.iter()
        .map(|f| &f.ty)
        .collect()
}