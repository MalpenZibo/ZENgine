extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Comma,
    DeriveInput, Ident, Index, LitInt, Path, Result,
};

mod zengine_manifest;
use zengine_manifest::ZENgineManifest;

pub(crate) fn zengine_ecs_path() -> syn::Path {
    ZENgineManifest::default().get_path("zengine_ecs")
}

#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let zengine_ecs_path: Path = crate::zengine_ecs_path();

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #zengine_ecs_path::component::Component for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Resource)]
pub fn resource_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let zengine_ecs_path: Path = crate::zengine_ecs_path();

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #zengine_ecs_path::world::Resource for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(InputType)]
pub fn input_type_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let zengine_input_path = ZENgineManifest::default().get_path("zengine_input");

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #zengine_input_path::InputType for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(SpriteType)]
pub fn sprite_type_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let zengine_graphic_path = ZENgineManifest::default().get_path("zengine_graphic");

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #zengine_graphic_path::SpriteType for #name #ty_generics #where_clause {}
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
pub fn all_tuples_with_idexes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuples);
    let len = input.end - input.start;
    let mut ident_tuples = Vec::with_capacity(len);
    for i in input.start..=input.end {
        let ident = format_ident!("{}{}", input.ident, i);
        let position: Index = syn::Index {
            index: i as u32,
            span: Span::call_site(),
        };
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

struct ZipInput {
    end: usize,
}

impl Parse for ZipInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let end = input.parse::<LitInt>()?.base10_parse()?;

        Ok(ZipInput { end })
    }
}

#[proc_macro]
pub fn generate_zip(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ZipInput);
    let mut expanded = quote! {};

    for zip_number in 3..input.end {
        let name = format_ident!("Zip{}", zip_number);

        let identity = format_ident!("Z{}", 0_usize);
        let identity_lower = format_ident!("z{}", 0_usize);

        let mut generics_with_where = quote!( #identity: Iterator );
        let mut generics = quote!( #identity );
        let mut generics_args = quote!( #identity_lower: #identity );
        for i in 1..zip_number {
            let identity = format_ident!("Z{}", i);
            let identity_lower = format_ident!("z{}", i);

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
        zip_type.extend(quote! { Z0, Z1> });

        for i in 2..zip_number {
            let identity = format_ident!("Z{}", i);
            zip_type.extend(quote! { , #identity > });
        }

        let identity1_lower = format_ident!("z{}", 0_usize);
        let identity2_lower = format_ident!("z{}", 1_usize);
        let mut zip_constructor = quote! { std::iter::zip(#identity1_lower, #identity2_lower) };
        for i in 2..zip_number {
            let identity_lower = format_ident!("z{}", i);
            zip_constructor = quote! { std::iter::zip(#zip_constructor, #identity_lower) };
        }

        let mut map_constructor_args = quote! { (#identity1_lower, #identity2_lower) };
        for i in 2..zip_number {
            let identity_lower = format_ident!("z{}", i);
            map_constructor_args = quote! { (#map_constructor_args, #identity_lower) };
        }

        let identity_lower = format_ident!("z{}", 0_usize);
        let mut map_constructor_res = quote! { #identity_lower };
        for i in 1..zip_number {
            let identity_lower = format_ident!("z{}", i);
            map_constructor_res.extend(quote! { , #identity_lower });
        }

        let identity = format_ident!("Z{}", 0_usize);
        let mut iter_output = quote! { #identity ::Item };
        for i in 1..zip_number {
            let identity = format_ident!("Z{}", i);
            iter_output.extend(quote! { , #identity ::Item });
        }

        let map_constructor = quote! { |#map_constructor_args| ( #map_constructor_res ) };

        expanded.extend(quote! {
            pub struct #name #generics_with_where {
                inner: #zip_type,
            }

            impl #generics_with_where #name #generics {
                #[allow(clippy::too_many_arguments)]
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

struct QueryIterInput {
    end: usize,
}

impl Parse for QueryIterInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let end = input.parse::<LitInt>()?.base10_parse()?;

        Ok(QueryIterInput { end })
    }
}

#[proc_macro]
pub fn query_iter_for_tuple(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryIterInput);
    let mut expanded = quote! {};

    expanded.extend(quote!{
        impl<'a, 'b, Z: QueryParameter> QueryIter<'b> for Query<'a, (Z,)>
        where
            <<Z as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIter<'b>,
        {
            type Iter = QueryIterator<
            <<<Z as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIter<
            'b,
        >>::Iter
            >;
            fn iter(&'b self) -> Self::Iter {
                QueryIterator::new(self.data.iter().map(|a| a.iter()).collect())
            }
        }
    });

    expanded.extend(quote!{
        impl<'a, 'b, Z0: QueryParameter, Z1: QueryParameter> QueryIter<'b> for Query<'a, (Z0, Z1)>
            where
                <<Z0 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIter<'b>,
                <<Z1 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIter<'b>,
            {
                type Iter = QueryIterator<
                    Zip<
                        <<<Z0 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIter<
                            'b,
                        >>::Iter,
                        <<<Z1 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIter<
                            'b,
                        >>::Iter,
                    >,
                >;
                fn iter(&'b self) -> Self::Iter {
                    QueryIterator::new(
                        self.data
                            .iter()
                            .map(|(z0, z1)| zip(z0.iter(), z1.iter()))
                            .collect(),
                    )
                }
            }
    });

    for zip_number in 3..input.end {
        let zip_type = format_ident!("Zip{}", zip_number);

        let identity = format_ident!("Z{}", 0_usize);
        let identity_lowercase = format_ident!("z{}", 0_usize);
        let mut generics = quote! { #identity: QueryParameter };
        let mut tuple = quote! { #identity };
        let mut where_clause = quote! { <<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIter<'b> };
        let mut zip_args = quote! { <<<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIter<'b,>>::Iter };
        let mut tuple_args = quote! { #identity_lowercase };
        let mut tuple_iter = quote! { #identity_lowercase.iter() };
        for i in 1..zip_number {
            let identity = format_ident!("Z{}", i);
            let identity_lowercase = format_ident!("z{}", i);
            generics.extend(quote! { , #identity: QueryParameter });
            tuple.extend(quote! { , #identity });
            where_clause.extend(quote! { , <<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIter<'b> });
            zip_args.extend(quote! { , <<<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIter<'b,>>::Iter });
            tuple_args.extend(quote! { , #identity_lowercase });
            tuple_iter.extend(quote! { , #identity_lowercase.iter() });
        }

        expanded.extend(quote! {
            impl<'a, 'b, #generics> QueryIter<'b> for Query<'a, ( #tuple )>
        where #where_clause
        {
            type Iter = QueryIterator<
                #zip_type<#zip_args>,
            >;
            fn iter(&'b self) -> Self::Iter {
                QueryIterator::new(
                    self.data
                        .iter()
                        .map(|( #tuple_args )| #zip_type::new( #tuple_iter ))
                        .collect(),
                )
            }
        }
        })
    }

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn query_iter_mut_for_tuple(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryIterInput);
    let mut expanded = quote! {};

    expanded.extend(quote!{
        impl<'a, 'b, Z: QueryParameter> QueryIterMut<'b> for Query<'a, (Z,)>
        where
            <<Z as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIterMut<'b>,
        {
            type Iter = QueryIterator<
            <<<Z as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIterMut<
            'b,
        >>::Iter
            >;
            fn iter_mut(&'b mut self) -> Self::Iter {
                QueryIterator::new(self.data.iter_mut().map(|a| a.iter_mut()).collect())
            }
        }
    });

    expanded.extend(quote!{
        impl<'a, 'b, Z0: QueryParameter, Z1: QueryParameter> QueryIterMut<'b> for Query<'a, (Z0, Z1)>
            where
                <<Z0 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIterMut<'b>,
                <<Z1 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIterMut<'b>,
            {
                type Iter = QueryIterator<
                    Zip<
                        <<<Z0 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIterMut<
                            'b,
                        >>::Iter,
                        <<<Z1 as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIterMut<
                            'b,
                        >>::Iter,
                    >,
                >;
                fn iter_mut(&'b mut self) -> Self::Iter {
                    QueryIterator::new(
                        self.data
                            .iter_mut()
                            .map(|(z0, z1)| zip(z0.iter_mut(), z1.iter_mut()))
                            .collect(),
                    )
                }
            }
    });

    for zip_number in 3..input.end {
        let zip_type = format_ident!("Zip{}", zip_number);

        let identity = format_ident!("Z{}", 0_usize);
        let identity_lowercase = format_ident!("z{}", 0_usize);
        let mut generics = quote! { #identity: QueryParameter };
        let mut tuple = quote! { #identity };
        let mut where_clause = quote! { <<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIterMut<'b> };
        let mut zip_args = quote! { <<<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIterMut<'b,>>::Iter };
        let mut tuple_args = quote! { #identity_lowercase };
        let mut tuple_iter = quote! { #identity_lowercase.iter_mut() };
        for i in 1..zip_number {
            let identity = format_ident!("Z{}", i);
            let identity_lowercase = format_ident!("z{}", i);
            generics.extend(quote! { , #identity: QueryParameter });
            tuple.extend(quote! { , #identity });
            where_clause.extend(quote! { , <<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem: QueryIterMut<'b> });
            zip_args.extend(quote! { , <<<#identity as QueryParameter>::Item as QueryParameterFetchFromArchetype<'a>>::ArchetypeFetchItem as QueryIterMut<'b,>>::Iter });
            tuple_args.extend(quote! { , #identity_lowercase });
            tuple_iter.extend(quote! { , #identity_lowercase.iter_mut() });
        }

        expanded.extend(quote! {
            impl<'a, 'b, #generics> QueryIterMut<'b> for Query<'a, ( #tuple )>
        where #where_clause
        {
            type Iter = QueryIterator<
                #zip_type<#zip_args>,
            >;
            fn iter_mut(&'b mut self) -> Self::Iter {
                QueryIterator::new(
                    self.data
                        .iter_mut()
                        .map(|( #tuple_args )| #zip_type::new( #tuple_iter ))
                        .collect(),
                )
            }
        }
        })
    }

    TokenStream::from(expanded)
}
