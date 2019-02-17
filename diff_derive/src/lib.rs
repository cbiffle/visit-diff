//! Derives the `Diff` trait naively, using the literal structure of the
//! datatype.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use std::iter::FromIterator;
use syn;
use syn::spanned::Spanned;

#[proc_macro_derive(Diff)]
pub fn diff_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let dispatch = gen_dispatch(&name, &input.data);

    let expanded = quote_spanned! {name.span()=>
        impl #impl_generics ::visit_diff::Diff for #name #ty_generics
        #where_clause {
            fn diff<D>(a: &Self, b: &Self, out: D)
                -> ::std::result::Result<D::Ok, D::Err>
            where D: ::visit_diff::Differ
            {
                #dispatch
            }
        }
    };

    TokenStream::from(expanded)
}

/// Naively slaps a `Diff` bound on every generic type parameter. This leads to
/// overconstrained impls but it's sure easy -- and it's essentially what the
/// built in derives do.
fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(syn::parse_quote!(::visit_diff::Diff));
        }
    }
    generics
}

/// Generates the "dispatcher" body of `diff`, which turns around and calls
/// methods on the `Differ` depending on type.
fn gen_dispatch(ty: &syn::Ident, data: &syn::Data) -> proc_macro2::TokenStream {
    match data {
        syn::Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => gen_named_struct(ty, fields),
                syn::Fields::Unnamed(fields) => gen_unnamed_struct(ty, fields),
                syn::Fields::Unit => {
                    // A unit struct without fields. There is only one instance
                    // of such a type, and so we know statically that our
                    // arguments are the same.
                    quote_spanned! {ty.span()=>
                        out.same(&a, &b)
                    }
                }
            }
        }
        syn::Data::Enum(data) => {
            // Enums are more complex than structs, because each variant can
            // have a different shape. We'll process the variants and generate
            // the corresponding match arms.
            let variants = data.variants.iter().map(|v| {
                let name = &v.ident;
                match &v.fields {
                    syn::Fields::Named(fields) => {
                        gen_named_variant(ty, name, fields)
                    }
                    syn::Fields::Unnamed(fields) => {
                        gen_unnamed_variant(ty, name, fields)
                    }
                    syn::Fields::Unit => {
                        // For a unit variant, we only need to check that both
                        // sides use the same variant.
                        quote_spanned! {v.span()=>
                            (#ty::#name, #ty::#name) => out.same(a, b),
                        }
                    }
                }
            });
            let variants = proc_macro2::TokenStream::from_iter(variants);

            // Now combine the match arms into a valid match expression.
            quote_spanned! {ty.span()=>
                match (a, b) {
                    #variants
                    _ => out.difference(a, b),
                }
            }
        }
        syn::Data::Union(_) => {
            unimplemented!("A `union` type cannot be meaningfully diffed")
        }
    }
}

/// Generates dispatcher for a named struct.
///
/// Named structs are different from enum variants with named fields, because of
/// the different ways we access their fields.
fn gen_named_struct(
    ty: &syn::Ident,
    fields: &syn::FieldsNamed,
) -> proc_macro2::TokenStream {
    // A traditional struct: named fields, curly braces, etc.
    // Generated code will resemble:
    //
    //   let mut s = out.begin_struct("TypeName");
    //   s.diff_field("field1", &a.field1, &b.field1);
    //   s.diff_field("field2", &a.field2, &b.field2);
    //   s.end()

    // First, generate the `diff_field` statements.
    let stmts = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {f.span()=>
            s.diff_field(stringify!(#name), &a.#name, &b.#name);
        }
    });
    let stmts = proc_macro2::TokenStream::from_iter(stmts);

    gen_named_struct_impl(ty, stmts)
}

/// Generates dispatcher for a named enum variant.
///
/// Named structs are different from enum variants with named fields, because of
/// the different ways we access their fields.
fn gen_named_variant(
    ty: &syn::Ident,
    name: &syn::Ident,
    fields: &syn::FieldsNamed,
) -> proc_macro2::TokenStream {
    // A variant with named fields is very much like a
    // struct, except that we have to access the fields
    // using pattern matching instead of dotted names.
    //
    // Generated match arm will resemble:
    //
    //   ( Ty::Var { f: f_a, v: v_a },
    //     Ty::Var { f: f_b, v: v_b } ) => {
    //       use ::visit_diff::StructDiffer;
    //       let mut s = out.begin_struct("Ty");
    //       s.diff_field("f", f_a, f_b);
    //       s.diff_field("v", v_a, v_b);
    //       s.end()
    //   },
    let a_pat = named_fields_pattern(fields.named.iter(), "_a");
    let b_pat = named_fields_pattern(fields.named.iter(), "_b");
    let stmts = diff_named_fields(fields.named.iter(), "_a", "_b");
    let walk = gen_named_struct_impl(name, stmts);
    quote_spanned! {name.span()=>
        ( #ty::#name { #a_pat },
          #ty::#name { #b_pat }) => {
            #walk
        },
    }
}

/// Common struct field walking code.
fn gen_named_struct_impl(
    ty: &syn::Ident,
    stmts: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote_spanned! {ty.span()=>
        use ::visit_diff::StructDiffer;
        let mut s = out.begin_struct(stringify!(#ty));
        #stmts
        s.end()
    }
}

/// Generates dispatcher for a struct with unnamed fields (i.e. a tuple struct).
fn gen_unnamed_struct(
    ty: &syn::Ident,
    fields: &syn::FieldsUnnamed,
) -> proc_macro2::TokenStream {
    // A tuple struct: unnamed fields, parens. Generated code
    // will resemble:
    //
    //   let mut s = out.begin_tuple("TypeName");
    //   s.diff_field(&a.0, &b.0);
    //   s.diff_field(&a.1, &b.1);
    //   s.end()

    // First, generate the `diff_field` statements.
    let stmts = fields.unnamed.iter().enumerate().map(|(i, f)| {
        let index = syn::Index::from(i);
        quote_spanned! {f.span()=>
            s.diff_field(&a.#index, &b.#index);
        }
    });
    let stmts = proc_macro2::TokenStream::from_iter(stmts);
    gen_unnamed_impl(ty, stmts)
}

/// Generates dispatcher for an enum variant with unnamed fields (i.e. a tuple
/// variant).
fn gen_unnamed_variant(
    ty: &syn::Ident,
    name: &syn::Ident,
    fields: &syn::FieldsUnnamed,
) -> proc_macro2::TokenStream {
    // A variant with unnamed fields is very much like a tuple struct, except
    // that we have to access the fields by pattern matching instead of using
    // dotted numbers.
    //
    // Generated match arm will resemble:
    //   ( Ty::Var(a0, a1),
    //     Ty::Var(b0, b1) ) => {
    //       use ::visit_diff::TupletDiffer;
    //       let mut s = out.begin_tuple("Ty");
    //       s.diff_field(f_a, f_b);
    //       s.diff_field(v_a, v_b);
    //       s.end()
    //   },
    let a_pat = unnamed_fields_pattern(fields.unnamed.iter(), "a");
    let b_pat = unnamed_fields_pattern(fields.unnamed.iter(), "b");
    let stmts = diff_unnamed_fields(fields.unnamed.iter(), "a", "b");
    let walk = gen_unnamed_impl(name, stmts);

    quote_spanned! {name.span()=>
        (#ty::#name(#a_pat), #ty::#name(#b_pat)) => {
            #walk
        },
    }
}

/// Common unnamed field walking code.
fn gen_unnamed_impl(
    ty: &syn::Ident,
    stmts: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote_spanned! {ty.span()=>
        use ::visit_diff::TupleDiffer;
        let mut s = out.begin_tuple(stringify!(#ty));
        #stmts
        s.end()
    }
}

/// Generates a pattern match that captures named fields under new names. This
/// is used to capture the values of fields in a named-field enum variant.
///
/// Because we are matching *two copies* of the variant, we can't use the simple
/// struct field match syntax, as it would try to bind each name twice:
///
/// ```ignore
/// (Variant { a, b, c }, Variant { a, b, c })
/// ```
///
/// Instead, we use this function to generate newly named bindings for each
/// field, suffixed by `suffix` -- which is different for the left and right
/// side. So, if we used the suffix `_left` on one side and `_right` on the
/// other, we'd get the following:
///
/// ```ignore
/// (Variant { a: a_left, b: b_left, c: c_left },
///  Variant { a: a_right, b: b_right, c: c_right })
/// ```
///
/// (This function is only responsible for the portion *within* the curly braces
/// above.)
fn named_fields_pattern<'a, I>(
    fields: I,
    suffix: &str,
) -> proc_macro2::TokenStream
where
    I: IntoIterator<Item = &'a syn::Field>,
{
    let pat = fields.into_iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let suffixed =
            syn::Ident::new(&format!("{}{}", name, suffix), name.span());
        quote_spanned! {f.span()=> #name: #suffixed, }
    });
    proc_macro2::TokenStream::from_iter(pat)
}

/// Generates a pattern match that gives names to unnamed fields. This is used
/// to capture the values of fields in a tuple-style enum variant.
///
/// We simply append a number to the given `prefix` for each field. So, if we
/// used the prefix `left_` on one side and `right_` on the other, a three-field
/// tuple enum variant would produce the following match pattern:
///
/// ```ignore
/// (Variant(left_0, left_1, left_2), Variant(right_0, right_1, right_2))
/// ```
///
/// (This function is only responsible for the portion *within* the inner
/// parentheses above.)
fn unnamed_fields_pattern<'a, I>(
    fields: I,
    prefix: &str,
) -> proc_macro2::TokenStream
where
    I: IntoIterator<Item = &'a syn::Field>,
{
    let pat = fields.into_iter().enumerate().map(|(i, f)| {
        let name = syn::Ident::new(&format!("{}{}", prefix, i), f.span());
        quote_spanned! {f.span()=> #name, }
    });
    proc_macro2::TokenStream::from_iter(pat)
}

/// Given named fields bound by `named_fields_pattern`, generates code to apply
/// the `StructDiffer` to each pair.
fn diff_named_fields<'a, I>(
    fields: I,
    left_suffix: &str,
    right_suffix: &str,
) -> proc_macro2::TokenStream
where
    I: IntoIterator<Item = &'a syn::Field>,
{
    let stmts = fields.into_iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let left =
            syn::Ident::new(&format!("{}{}", name, left_suffix), name.span());
        let right =
            syn::Ident::new(&format!("{}{}", name, right_suffix), name.span());
        quote_spanned! {f.span()=>
            s.diff_field(stringify!(#name), #left, #right);
        }
    });
    proc_macro2::TokenStream::from_iter(stmts)
}

/// Given unnamed fields bound by `unnamed_fields_pattern`, generates code to
/// apply the `TupleDiffer` to each pair.
fn diff_unnamed_fields<'a, I>(
    fields: I,
    left_prefix: &str,
    right_prefix: &str,
) -> proc_macro2::TokenStream
where
    I: IntoIterator<Item = &'a syn::Field>,
{
    let stmts = fields.into_iter().enumerate().map(|(i, f)| {
        let left = syn::Ident::new(&format!("{}{}", left_prefix, i), f.span());
        let right =
            syn::Ident::new(&format!("{}{}", right_prefix, i), f.span());
        quote_spanned! {f.span()=>
            s.diff_field(#left, #right);
        }
    });
    proc_macro2::TokenStream::from_iter(stmts)
}
