use super::*;

pub fn ssz_encode_derive(item_ast: &syn::DeriveInput) -> TokenStream {
    let name = &item_ast.ident;
    let (impl_generics, type_generics, where_clause) = &item_ast.generics.split_for_impl();

    let struct_meta = match &item_ast.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Encode derive macro supports only structs")
    };

    let mut is_ssz_fixed_lens = vec![];
    let mut ssz_fixed_lens = vec![];
    let mut appends = vec![];
    let mut ssz_bytes_lens = vec![];

    struct_meta.fields.iter()
        .filter(|field| should_serialize_field(*field))
        .for_each(|field| {
            let field_type = &field.ty;
            let field_name = extract_ident(field);

            is_ssz_fixed_lens.push(quote! {
                <#field_type as ssz::Encode>::is_ssz_fixed_len()
            });

            ssz_fixed_lens.push(quote! {
                <#field_type as ssz::Encode>::ssz_fixed_len()
            });

            appends.push(quote! {
                encoder.append(&self.#field_name)
            });

            ssz_bytes_lens.push(quote! {
                len += if <#field_type as ssz::Encode>::is_ssz_fixed_len() {
                    <#field_type as ssz::Encode>::ssz_fixed_len()
                } else {
                    self.#field_name.ssz_bytes_len() + ssz::BYTES_PER_LENGTH_OFFSET
                }
            });
        });

    // we have to clone this because you can use vector only once in code generation
    let ssz_fixed_lens_2 = ssz_fixed_lens.clone();

    let generated = quote! {
        impl #impl_generics Encode for #name #type_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                #(
                    #is_ssz_fixed_lens &&
                )*
                    true
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                let offset =
                    #(
                        #ssz_fixed_lens +
                    )*
                        0;

                let mut encoder = ssz::SszEncoder::container(buf, offset);

                #(
                    #appends;
                )*

                encoder.finalize();
            }

            fn ssz_fixed_len() -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    #(
                        #ssz_fixed_lens_2 +
                    )*
                        0
                } else {
                    ssz::BYTES_PER_LENGTH_OFFSET
                }
            }

            fn ssz_bytes_len(&self) -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    <Self as ssz::Encode>::ssz_fixed_len()
                } else {
                     let mut len = 0;

                     #(
                        #ssz_bytes_lens;
                     )*

                     len
                }
            }
        }
    };

    generated.into()
}

pub fn ssz_decode_derive(item_ast: &syn::DeriveInput) -> TokenStream {
    let name = &item_ast.ident;
    let (impl_generics, type_generics, where_clause) = &item_ast.generics.split_for_impl();

    let struct_meta = match &item_ast.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Decode derive macro supports only structs")
    };

    let mut is_ssz_fixed_lens = vec![];
    let mut ssz_fixed_lens = vec![];
    let mut register_types = vec![];
    let mut struct_fields = vec![];

    struct_meta.fields.iter()
        .for_each(|field| {
            let field_type = &field.ty;
            let field_name = extract_ident(field);

            if should_deserialize_field(field) {
                is_ssz_fixed_lens.push(quote! {
                    <#field_type as ssz::Decode>::is_ssz_fixed_len()
                });

                ssz_fixed_lens.push(quote! {
                    <#field_type as ssz::Decode>::ssz_fixed_len()
                });

                register_types.push(quote! {
                    builder.register_type::<#field_type>()?
                });

                struct_fields.push(quote! {
                    #field_name: decoder.decode_next()?
                });
            } else {
                struct_fields.push(quote! {
                    #field_name: <_>::default()
                });
            }
        });

    let generated = quote! {
        impl #impl_generics ssz::Decode for #name #type_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                #(
                    #is_ssz_fixed_lens &&
                )*
                    true
            }

            fn ssz_fixed_len() -> usize {
                if <Self as ssz::Decode>::is_ssz_fixed_len() {
                    #(
                        #ssz_fixed_lens +
                    )*
                        0
                } else {
                    ssz::BYTES_PER_LENGTH_OFFSET
                }
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
                let mut builder = ssz::SszDecoderBuilder::new(bytes);

                #(
                    #register_types;
                )*

                let mut decoder = builder.build()?;

                Ok(Self {
                    #(
                        #struct_fields,
                    )*
                })
            }
        }
    };

    generated.into()
}

fn extract_ident(field: &syn::Field) -> &syn::Ident {
    match &field.ident {
        Some(ident) => ident,
        _ => panic!("Decoding supports only named fields")
    }
}

fn should_deserialize_field(field: &syn::Field) -> bool {
    !field.attrs.iter()
        .any(|attr|
            attr.path.is_ident("ssz")
                && attr.tts.to_string().replace(" ", "") == "(skip_deserializing)")
}

fn should_serialize_field(field: &syn::Field) -> bool {
    !field.attrs.iter()
        .any(|attr|
            attr.path.is_ident("ssz")
                && attr.tts.to_string().replace(" ", "") == "(skip_serializing)")
}