extern crate proc_macro;

use std::iter::FromIterator;
use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::spanned::Spanned;

#[proc_macro_derive(Diff)]
pub fn diff_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let dispatch = gen_dispatch(&name, &input.data);

    let expanded = quote! {
        impl #impl_generics ::diffwalk::Diff for #name #ty_generics
        #where_clause {
            fn diff<D>(a: &Self, b: &Self, out: D)
                -> ::std::result::Result<D::Ok, D::Err>
            where D: ::diffwalk::Differ
            {
                #dispatch
            }
        }
    };

    TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param.bounds.push(syn::parse_quote!(::diffwalk::Diff));
        }
    }
    generics
}

fn gen_dispatch(ty: &syn::Ident, data: &syn::Data) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => {
                    let stmts = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote::quote_spanned! {f.span()=>
                            s.diff_field(stringify!(#name), &a.#name, &b.#name);
                        }
                    });
                    let stmts = proc_macro2::TokenStream::from_iter(stmts);
                    quote! {
                        use ::diffwalk::StructDiffer;
                        let mut s = out.begin_struct(stringify!(#ty));
                        #stmts
                        s.end()
                    }
                },
                syn::Fields::Unnamed(fields) => {
                    let stmts = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let index = syn::Index::from(i);
                        quote::quote_spanned! {f.span()=>
                            s.diff_field(&a.#index, &b.#index);
                        }
                    });
                    let stmts = proc_macro2::TokenStream::from_iter(stmts);
                    quote! {
                        use ::diffwalk::TupleDiffer;
                        let mut s = out.begin_tuple(stringify!(#ty));
                        #stmts
                        s.end()
                    }
                },
                syn::Fields::Unit => {
                    quote! {
                        out.same(&a, &b)
                    }
                },
            }
        },
        syn::Data::Enum(_) => {
            unimplemented!()
        },
        syn::Data::Union(_) => {
            unimplemented!()
        },
    }
}
