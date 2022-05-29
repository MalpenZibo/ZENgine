extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Comma,
    DeriveInput, Ident, Index, LitInt, Result,
};

#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Component for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Resource)]
pub fn resource_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Resource for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(InputType)]
pub fn input_type_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics InputType for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(SpriteType)]
pub fn sprite_type_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics SpriteType for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

struct AllTuples {
    macro_ident: Ident,
    start: usize,
    end: usize,
    ident: Ident,
}

impl Parse for AllTuples {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let start = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let end = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let ident = input.parse::<Ident>()?;

        Ok(AllTuples {
            macro_ident,
            start,
            end,
            ident,
        })
    }
}

#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuples);
    let len = input.end - input.start;
    let mut ident_tuples = Vec::with_capacity(len);
    for i in input.start..=input.end {
        let ident = format_ident!("{}{}", input.ident, i);
        ident_tuples.push(quote! {
            #ident
        });
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let ident_tuples = &ident_tuples[0..i - input.start];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}

#[proc_macro]
pub fn all_positional_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuples);
    let len = input.end - input.start;
    let mut ident_tuples = Vec::with_capacity(len);
    for i in input.start..=input.end {
        let position: Index = syn::Index {
            index: i as u32,
            span: Span::call_site(),
        };
        let ident = format_ident!("{}{}", input.ident, i);
        ident_tuples.push(quote! {
            #ident => #position
        });
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let ident_tuples = &ident_tuples[0..i - input.start];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });

    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}

#[proc_macro]
pub fn generate_zip(_item: TokenStream) -> TokenStream {
    let a: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string().chars().collect();
    let mut expanded = quote! {};

    for zip_number in 3..26 {
        let name = syn::Ident::new(&format!("Zip{}", zip_number), Span::call_site());

        let identity = syn::Ident::new(&a[0].to_string(), Span::call_site());
        let identity_lower = syn::Ident::new(&a[0].to_string().to_lowercase(), Span::call_site());

        let mut generics_with_where = quote!( #identity: Iterator );
        let mut generics = quote!( #identity );
        let mut generics_args = quote!( #identity_lower: #identity );
        for i in 1..zip_number {
            let identity = syn::Ident::new(&a[i].to_string(), Span::call_site());
            let identity_lower =
                syn::Ident::new(&a[i].to_string().to_lowercase(), Span::call_site());

            generics_with_where.extend(quote! { , #identity: Iterator });
            generics.extend(quote! { , #identity });
            generics_args.extend(quote! { , #identity_lower: #identity })
        }

        let generics = quote!( < #generics > );
        let generics_with_where = quote!( < #generics_with_where > );

        let mut zip_type = quote! {};
        for _i in 0..zip_number - 1 {
            zip_type.extend(quote! { std::iter::Zip< });
        }
        zip_type.extend(quote! { A, B> });

        for i in 2..zip_number {
            let identity = syn::Ident::new(&a[i].to_string(), Span::call_site());
            zip_type.extend(quote! { , #identity > });
        }

        let identity1_lower = syn::Ident::new(&a[0].to_string().to_lowercase(), Span::call_site());
        let identity2_lower = syn::Ident::new(&a[1].to_string().to_lowercase(), Span::call_site());
        let mut zip_constructor = quote! { std::iter::zip(#identity1_lower, #identity2_lower) };

        for i in 2..zip_number {
            let identity_lower =
                syn::Ident::new(&a[i].to_string().to_lowercase(), Span::call_site());
            zip_constructor = quote! { std::iter::zip(#zip_constructor, #identity_lower) };
        }

        let identity1_lower = syn::Ident::new(&a[0].to_string().to_lowercase(), Span::call_site());
        let identity2_lower = syn::Ident::new(&a[1].to_string().to_lowercase(), Span::call_site());
        let mut map_constructor_args = quote! { (#identity1_lower, #identity2_lower) };

        for i in 2..zip_number {
            let identity_lower =
                syn::Ident::new(&a[i].to_string().to_lowercase(), Span::call_site());
            map_constructor_args = quote! { (#map_constructor_args, #identity_lower) };
        }
        let mut map_constructor_res = quote! {};
        for i in 0..zip_number {
            let identity_lower =
                syn::Ident::new(&a[i].to_string().to_lowercase(), Span::call_site());
            if i == zip_number - 1 {
                map_constructor_res.extend(quote! { #identity_lower });
            } else {
                map_constructor_res.extend(quote! { #identity_lower, });
            }
        }

        let mut iter_output = quote! {};
        for i in 0..zip_number {
            let identity = syn::Ident::new(&a[i].to_string(), Span::call_site());
            if i == zip_number - 1 {
                iter_output.extend(quote! { #identity ::Item });
            } else {
                iter_output.extend(quote! { #identity ::Item, });
            }
        }

        let map_constructor = quote! { |#map_constructor_args| ( #map_constructor_res ) };

        expanded.extend(quote! {
            pub struct #name #generics_with_where {
                inner: #zip_type,
            }

            impl #generics_with_where #name #generics {
                #[allow(non_snake_case)]
                pub fn new (#generics_args) -> Self {
                    Self {
                        inner: #zip_constructor
                    }
                }
            }

            impl #generics_with_where Iterator for #name #generics {
                type Item = (#iter_output);

                #[inline(always)]
                fn next(&mut self) -> Option<Self::Item> {
                    self.inner.next().map(#map_constructor)
                }
                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.inner.size_hint()
                }
            }

        });
    }

    TokenStream::from(expanded)
}
