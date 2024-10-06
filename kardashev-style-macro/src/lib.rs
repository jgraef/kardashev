#![feature(track_path)]

use std::path::PathBuf;

use darling::{
    ast::NestedMeta,
    FromMeta,
};
use heck::ToSnekCase;
use proc_macro2::{
    Span,
    TokenStream,
};
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    Ident,
    ItemStruct,
    LitStr,
};

#[proc_macro_attribute]
pub fn style(
    attributes: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match style_impl(item.into(), attributes.into()) {
        Ok(output) => output,
        Err(error) => error.write_errors(),
    }
    .into()
}

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(rename = "path")]
    input_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Error while parsing macro input")]
    Syn(#[from] syn::Error),
    #[error("Error while parsing attribute list")]
    Darling(#[from] darling::Error),
    #[error("{0}")]
    Transform(#[from] kardashev_style_internal::Error),
}

impl Error {
    pub fn write_errors(self) -> TokenStream {
        match self {
            Self::Darling(error) => error.write_errors(),
            error => {
                let mut current_error: &dyn std::error::Error = &error;

                loop {
                    eprintln!("{current_error}");
                    if let Some(e) = current_error.source() {
                        current_error = e;
                    }
                    else {
                        break;
                    }
                }

                panic!("{error}");
            }
        }
    }
}

fn style_impl(item: TokenStream, attributes: TokenStream) -> Result<TokenStream, Error> {
    let span = Span::call_site();
    let item = syn::parse2::<ItemStruct>(item)?;

    let args = NestedMeta::parse_meta_list(attributes)?;
    let args = MacroArgs::from_list(&args)?;

    let output = kardashev_style_internal::prepare_import(&args.input_path, |path| {
        proc_macro::tracked_path::path(path.to_str().expect("failed to convert path to string"));
    })?;

    let ident = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    let mut const_names = vec![];
    let mut class_names = vec![];
    for (original_class_name, mangled_class_name) in &output.class_names {
        const_names.push(Ident::new(&original_class_name.to_snek_case(), span));
        class_names.push(LitStr::new(&mangled_class_name, span));
    }

    let impl_block = quote_spanned! {
        span =>
        #[allow(non_upper_case_globals)]
        impl #impl_generics #ident #type_generics #where_clause {
            #(
                const #const_names: &'static str = #class_names;
            )*
        }
    };

    Ok(quote! {
        #item
        #impl_block
    })
}
