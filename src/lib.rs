#![allow(unused_macros, unused_imports, unused_variables, dead_code)]
//! # Typed Builder
//!
//! This crate provides a custom derive for `TypedBuilder`. `TypedBuilder` is not a real type -
//! deriving it will generate a `::builder()` method on your struct that will return a compile-time
//! checked builder. Set the fields using setters with the same name as the struct's fields that
//! accept `Into` types for the type of the field, and call `.build()` when you are done to create
//! your object.
//!
//! Trying to set the same fields twice will generate a compile-time error. Trying to build without
//! setting one of the fields will also generate a compile-time error - unless that field is marked
//! as `#[default]`, in which case the `::default()` value of it's type will be picked. If you want
//! to set a different default, use `#[default="..."]` - note that it has to be encoded in a
//! string, so `1` is `#[default="1"]` and `"hello"` is `#[default="\"hello\""]`.
//!
//! # Examples
//!
//! ```
//! #[macro_use]
//! extern crate typed_builder;
//!
//! #[derive(PartialEq, TypedBuilder)]
//! struct Foo {
//!     // Mandatory Field:
//!     x: i32,
//!
//!     // #[default] without parameter - use the type's default
//!     #[builder(default)]
//!     y: Option<i32>,
//!
//!     // Or you can set the default(encoded as string)
//!     #[builder(default=20)]
//!     z: i32,
//! }
//!
//! fn main() {
//!     assert!(
//!         Foo::builder().x(1).y(2).z(3).build()
//!         == Foo { x: 1, y: Some(2), z: 3 });
//!
//!     // Change the order of construction:
//!     assert!(
//!         Foo::builder().z(1).x(2).y(3).build()
//!         == Foo { x: 2, y: Some(3), z: 1 });
//!
//!     // Optional fields are optional:
//!     assert!(
//!         Foo::builder().x(1).build()
//!         == Foo { x: 1, y: None, z: 20 });
//!
//!     // This will not compile - because we did not set x:
//!     // Foo::builder().build();
//!
//!     // This will not compile - because we set y twice:
//!     // Foo::builder().x(1).y(2).y(3);
//! }
//! ```
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

extern crate quote;

use proc_macro2::TokenStream;

use syn::{
    parse_macro_input,
    DeriveInput,
};
use syn::parse::Error;
use syn::spanned::Spanned;

use quote::quote;

mod util;
mod field_info;
mod struct_info;
mod builder_attr;


#[doc(hidden)]
#[proc_macro_derive(TypedBuilder, attributes(builder))]
pub fn derive_typed_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_my_derive(&input) {
        Ok(output) => {
            output.into()
        },
        Err(error) =>{
            error.to_compile_error().into()
        }
    }
}

fn impl_my_derive(ast: &syn::DeriveInput) -> Result<TokenStream, Error> {
    let data = match &ast.data {
        syn::Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => {
                    let struct_info = struct_info::StructInfo::new(&ast, fields.named.iter())?;
                    let builder_creation = struct_info.builder_creation_impl()?;
                    // println!("builder_creation\n==============\n{}\n==============", builder_creation);
                    let conversion_helper = struct_info.conversion_helper_impl()?;
                    // println!("conversion_helper\n==============\n{}\n==============", conversion_helper);
                    let fields = struct_info.fields.iter().map(|f| struct_info.field_impl(f).unwrap());
                    // for field in fields {
                        // println!("field\n==============\n{}\n==============", quote!(#field));
                    // }
                    let fields = quote!(#(#fields)*);
                    // println!("Fields be {}", fields);
                    // println!("fields\n==============\n{}\n==============", fields);
                    let build_method = struct_info.build_method_impl();

                    // eprintln!("===\n{}\n===", builder_creation);
                    // println!("{}", builder_creation);
                    quote!{
                        #builder_creation
                        #conversion_helper
                        #( #fields )*
                        #build_method
                    }
                }
                syn::Fields::Unnamed(_) => return Err(Error::new(ast.span(), "SmartBuilder is not supported for tuple structs")),
                syn::Fields::Unit => return Err(Error::new(ast.span(), "SmartBuilder is not supported for unit structs")),
            }
        }
        syn::Data::Enum(_) => return Err(Error::new(ast.span(), "SmartBuilder is not supported for enums")),
        syn::Data::Union(_) => return Err(Error::new(ast.span(), "SmartBuilder is not supported for unions")),
        // syn::Data::Struct(syn::VariantData::Struct(ref body)) => {
            // let struct_info = struct_info::StructInfo::new(&ast, body);
            // let builder_creation = struct_info.builder_creation_impl();
            // let conversion_helper = struct_info.conversion_helper_impl();
            // let fields = struct_info.fields.iter().map(|f| struct_info.field_impl(f));
            // let build_method = struct_info.build_method_impl();
            // quote!{
                // #builder_creation
                // #conversion_helper
                // #( #fields )*
                // #build_method
            // }
        // },
        // syn::Data::Struct(syn::VariantData::Unit) => panic!("SmartBuilder is not supported for unit types"),
        // syn::Data::Struct(syn::VariantData::Tuple(_)) => panic!("SmartBuilder is not supported for tuples"),
        // syn::Data::Enum(_) => panic!("SmartBuilder is not supported for enums"),
    };
    // println!("{}", data);
    Ok(data)
    // Ok(quote!())
}
// fn impl_my_derive(ast: &syn::DeriveInput) -> TokenStream {

    // match ast.body {
        // syn::Body::Struct(syn::VariantData::Struct(ref body)) => {
            // let struct_info = struct_info::StructInfo::new(&ast, body);
            // let builder_creation = struct_info.builder_creation_impl();
            // let conversion_helper = struct_info.conversion_helper_impl();
            // let fields = struct_info.fields.iter().map(|f| struct_info.field_impl(f));
            // let build_method = struct_info.build_method_impl();
            // quote!{
                // #builder_creation
                // #conversion_helper
                // #( #fields )*
                // #build_method
            // }
        // },
        // syn::Body::Struct(syn::VariantData::Unit) => panic!("SmartBuilder is not supported for unit types"),
        // syn::Body::Struct(syn::VariantData::Tuple(_)) => panic!("SmartBuilder is not supported for tuples"),
        // syn::Body::Enum(_) => panic!("SmartBuilder is not supported for enums"),
    // }
// }
