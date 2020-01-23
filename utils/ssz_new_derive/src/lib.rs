#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields};

#[proc_macro_derive(Encode, attributes(ssz))]
pub fn encode_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("AST should be correct");

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let fields = get_serializable_fields(&ast.data);

    let fields_count = fields.iter().len();

    let mut fixed_parts_pushes = Vec::with_capacity(fields_count);
    let mut variable_parts_pushes = Vec::with_capacity(fields_count);
    let mut is_fixed_lens = Vec::with_capacity(fields_count);
    let mut ssz_bytes_lens = Vec::with_capacity(fields_count);
    let mut ssz_fixed_lens = Vec::with_capacity(fields_count);
    for field in fields {
        let field_type = &field.ty;
        let field_name = match &field.ident {
            Some(ident) => ident,
            _ => panic!("All fields must have names"),
        };

        fixed_parts_pushes.push(quote! {
            fixed_parts.push(if <#field_type as ssz::Encode>::is_ssz_fixed_len() {
                Some(self.#field_name.as_ssz_bytes())
            } else {
                None
            });
        });

        variable_parts_pushes.push(quote! {
            variable_parts.push(if <#field_type as ssz::Encode>::is_ssz_fixed_len() {
                vec![]
            } else {
                self.#field_name.as_ssz_bytes()
            });
        });

        is_fixed_lens.push(quote! {
            <#field_type as ssz::Encode>::is_ssz_fixed_len()
        });

        ssz_bytes_lens.push(quote! {
            self.#field_name.ssz_bytes_len()
        });

        ssz_fixed_lens.push(quote! {
            <#field_type as ssz::Encode>::ssz_fixed_len()
        });
    }

    let generated = quote! {
        impl #impl_generics ssz::Encode for #name #ty_generics #where_clause {
            fn ssz_append(&self, buf: &mut Vec<u8>) {
                let fields_count = #fields_count;

                let mut fixed_parts = Vec::with_capacity(fields_count);
                #(
                    #fixed_parts_pushes
                )*

                let mut variable_parts = Vec::with_capacity(fields_count);
                #(
                    #variable_parts_pushes
                )*

                ssz::encode_items_from_parts(buf, fixed_parts, variable_parts);
            }

            fn is_ssz_fixed_len() -> bool {
                #(
                    #is_fixed_lens &&
                )*
                    true
            }

            fn ssz_bytes_len(&self) -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    <Self as ssz::Encode>::ssz_fixed_len()
                } else {
                    #(
                        #ssz_bytes_lens +
                    )*
                        ssz::BYTES_PER_LENGTH_OFFSET
                }
            }

            fn ssz_fixed_len() -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    #(
                        #ssz_fixed_lens +
                    )*
                        0
                } else {
                    ssz::BYTES_PER_LENGTH_OFFSET
                }
            }
        }
    };

    generated.into()
}

#[proc_macro_derive(Decode, attributes(ssz))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("AST should be correct");

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let fields = get_deserializable_fields(&ast.data);

    let fields_count = fields.iter().len();

    let mut next_types = Vec::with_capacity(fields_count);
    let mut fields_initialization = Vec::with_capacity(fields_count);
    let mut is_fixed_lens = Vec::with_capacity(fields_count);
    let mut fixed_lengths = Vec::with_capacity(fields_count);
    for field in fields {
        let field_type = &field.ty;
        let field_name = match &field.ident {
            Some(ident) => ident,
            _ => panic!("All fields must have names"),
        };

        if should_ship_deserialization(field) {
            fields_initialization.push(quote! {
                #field_name: <_>::default()
            });
        } else {
            next_types.push(quote! {
                decoder.next_type::<#field_type>()?
            });

            fields_initialization.push(quote! {
                #field_name: decoder.deserialize_next::<#field_type>()?
            });

            is_fixed_lens.push(quote! {
                <#field_type as ssz::Decode>::is_ssz_fixed_len()
            });

            fixed_lengths.push(quote! {
               <#field_type as ssz::Decode>::ssz_fixed_len()
            });
        }
    }

    let generated = quote! {
        impl #impl_generics ssz::Decode for #name #ty_generics #where_clause {
            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
                let mut decoder = ssz::Decoder::for_bytes(bytes);

                #(
                    #next_types;
                )*

                Ok(Self {
                    #(
                        #fields_initialization,
                    )*
                })
            }

            fn is_ssz_fixed_len() -> bool {
                #(
                    #is_fixed_lens &&
                )*
                    true
            }

            fn ssz_fixed_len() -> usize {
                #(
                    #fixed_lengths +
                )*
                    0
            }
        }
    };

    generated.into()
}

fn get_serializable_fields(data: &Data) -> Vec<&Field> {
    extract_fields(data)
        .iter()
        .filter(|f| !should_ship_serialization(f))
        .collect()
}

fn get_deserializable_fields(data: &Data) -> Vec<&Field> {
    extract_fields(data).iter().collect()
}

fn should_ship_serialization(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path.is_ident("ssz") && attr.tts.to_string().replace(" ", "") == "(skip_serializing)"
    })
}

fn should_ship_deserialization(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path.is_ident("ssz") && attr.tts.to_string().replace(" ", "") == "(skip_deserializing)"
    })
}

fn extract_fields(data: &Data) -> &Fields {
    match data {
        syn::Data::Struct(struct_data) => &struct_data.fields,
        _ => panic!("Serialization only available for structs"),
    }
}
