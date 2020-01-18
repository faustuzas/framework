#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(SszSerialize)]
pub fn serialize_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("AST should be correct");

    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(struct_data) => &struct_data.fields,
        _ => panic!("Serialization only available for structs"),
    };
    let fields_count = fields.iter().len();

    let mut fixed_parts_pushes = Vec::with_capacity(fields_count);
    let mut variable_parts_pushes = Vec::with_capacity(fields_count);
    let mut is_variable_sizes = Vec::with_capacity(fields_count);
    for field in fields {
        let field_type = &field.ty;
        let field_name = match &field.ident {
            Some(ident) => ident,
            _ => panic!("All fields must have names"),
        };

        fixed_parts_pushes.push(quote! {
            fixed_parts.push(if !<#field_type as ssz::Serialize>::is_variable_size() {
                Some(self.#field_name.serialize()?)
            } else {
                None
            });
        });

        variable_parts_pushes.push(quote! {
            variable_parts.push(if <#field_type as ssz::Serialize>::is_variable_size() {
                self.#field_name.serialize()?
            } else {
                vec![]
            });
        });

        is_variable_sizes.push(quote! {
            <#field_type as ssz::Serialize>::is_variable_size()
        });
    }

    let generated = quote! {
        impl ssz::Serialize for #name {
            fn serialize(&self) -> Result<Vec<u8>, ssz::Error> {
                let fields_count = #fields_count;

                let mut fixed_parts = Vec::with_capacity(fields_count);
                #(
                    #fixed_parts_pushes
                )*

                let mut variable_parts = Vec::with_capacity(fields_count);
                #(
                    #variable_parts_pushes
                )*

                let fixed_length: usize = fixed_parts.iter()
                    .map(|part| match part {
                        Some(bytes) => bytes.len(),
                        None => ssz::BYTES_PER_LENGTH_OFFSET
                    }).sum();

                let variable_lengths: Vec<usize> = variable_parts.iter()
                    .map(|part| part.len())
                    .collect();

                let mut variable_offsets = Vec::with_capacity(fields_count);
                for i in 0..fields_count {
                    let variable_length_sum: usize = variable_lengths[..i].iter().sum();
                    let offset = fixed_length + variable_length_sum;
                    variable_offsets.push(ssz::serialize_offset(offset)?);
                }

                let fixed_parts: Vec<&Vec<u8>> = fixed_parts.iter()
                    .enumerate()
                    .map(|(i, part)| match part {
                        Some(bytes) => bytes,
                        None => &variable_offsets[i]
                    }).collect();

                let variable_lengths_sum: usize = variable_lengths.iter().sum();
                let total_bytes = fixed_length + variable_lengths_sum;
                let mut result = Vec::with_capacity(total_bytes);

                for part in fixed_parts {
                    result.extend(part);
                }

                for part in variable_parts {
                    result.extend(part);
                }

                Ok(result)
            }

            fn is_variable_size() -> bool {
                #(
                    #is_variable_sizes ||
                )*
                    false
            }
        }
    };

    generated.into()
}

#[proc_macro_derive(SszDeserialize)]
pub fn deserialize_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("AST should be correct");

    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(struct_data) => &struct_data.fields,
        _ => panic!("Deserialization only available for structs"),
    };
    let fields_count = fields.iter().len();

    let mut next_types = Vec::with_capacity(fields_count);
    let mut fields_initialization = Vec::with_capacity(fields_count);
    let mut is_variable_sizes = Vec::with_capacity(fields_count);
    let mut fixed_lengths = Vec::with_capacity(fields_count);
    for field in fields {
        let field_type = &field.ty;
        let field_name = match &field.ident {
            Some(ident) => ident,
            _ => panic!("All fields must have names"),
        };

        next_types.push(quote! {
            decoder.next_type::<#field_type>()?
        });

        fields_initialization.push(quote! {
            #field_name: decoder.deserialize_next::<#field_type>()?
        });

        is_variable_sizes.push(quote! {
            <#field_type as ssz::Deserialize>::is_variable_size()
        });

        fixed_lengths.push(quote! {
           <#field_type>::fixed_length()
        });
    }

    let generated = quote! {
        impl ssz::Deserialize for #name {
            fn deserialize(bytes: &[u8]) -> Result<Self, ssz::Error> {
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

            fn is_variable_size() -> bool {
                #(
                    #is_variable_sizes ||
                )*
                    false
            }

            fn fixed_length() -> usize {
                #(
                    #fixed_lengths +
                )*
                    0
            }
        }
    };

    generated.into()
}
