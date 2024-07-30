extern crate proc_macro;
use quote::quote;


fn replace_ident(stream: proc_macro2::TokenStream, indent_from: &str, indent_to: &str) -> proc_macro2::TokenStream {
    stream.into_iter().map(|token| {
        match token {
            proc_macro2::TokenTree::Group(group) => {
                let new_stream = replace_ident(group.stream(), indent_from, indent_to);
                proc_macro2::TokenTree::Group(proc_macro2::Group::new(group.delimiter(), new_stream))
            },
            proc_macro2::TokenTree::Ident(ident) => {
                if ident == indent_from {
                    proc_macro2::TokenTree::Ident(proc_macro2::Ident::new(indent_to, ident.span()))
                } else {
                    proc_macro2::TokenTree::Ident(ident)
                }
            },
            proc_macro2::TokenTree::Punct(punct) => proc_macro2::TokenTree::Punct(punct),
            proc_macro2::TokenTree::Literal(lit) => proc_macro2::TokenTree::Literal(lit),
        }
    }).collect()
}


#[proc_macro_attribute]
pub fn _async(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(input as syn::Item);
    let output = match item {
        syn::Item::Fn(mut item) => {
            let ident_async = proc_macro2::Ident::new(&format!("{}_async", item.sig.ident), item.sig.ident.span());
            let output_sync = quote! {
                #item
            };
            let output_sync = replace_ident(output_sync, "_await", "_await_sync");

            item.sig.asyncness = Some(syn::token::Async(item.sig.ident.span()));
            item.sig.ident = ident_async;
            let output_async = quote! {
                #item
            };
            let output_async = replace_ident(output_async, "_await", "_await_async");

            quote! {
                #output_sync 
                #output_async 
            }

        }
        syn::Item::Verbatim(input) => {
            let input = input.into();
            let mut item = syn::parse_macro_input!(input as syn::TraitItemFn);

            let ident_async = proc_macro2::Ident::new(&format!("{}_async", item.sig.ident), item.sig.ident.span());
            let output_sync = quote! {
                #item
            };
            let output_sync = replace_ident(output_sync, "_await", "_await_sync");

            item.sig.asyncness = Some(syn::token::Async(item.sig.ident.span()));
            item.sig.ident = ident_async;
            let output_async = quote! {
                #item
            };
            let output_async = replace_ident(output_async, "_await", "_await_async");

            quote! {
                #output_sync
                #output_async
            }
        }
        _ => panic!("The macro `#[_async]` can only be applied to a function definition."),
    };

    proc_macro::TokenStream::from(output)
}

#[proc_macro]
pub fn _await_sync(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(input)
}

#[proc_macro]
pub fn _await_async(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = match syn::parse_macro_input!(input as syn::Expr) {
        syn::Expr::MethodCall(mut item) => {
            let ident_async = proc_macro2::Ident::new(&format!("{}_async", item.method), item.method.span());
            item.method = ident_async;
            quote! {
                #item.await
            }
        }
        syn::Expr::Call(mut item) => {
            match &mut *item.func {
                syn::Expr::Path(syn::ExprPath {path: syn::Path{segments, ..}, ..}) => {
                    let last_segment = segments.last_mut().unwrap();
                    let ident_async = proc_macro2::Ident::new(&format!("{}_async", last_segment.ident), last_segment.ident.span());
                    last_segment.ident = ident_async;
                }
                _ => panic!("The macro `_await!` can only be applied to a method or explicit function call."),
            }
            quote! {
                #item.await
            }
        }
        _ => panic!("The macro `_await!` can only be applied to a function or method call."),
    };
    proc_macro::TokenStream::from(output)
}
