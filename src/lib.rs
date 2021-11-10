use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{PathArguments, ReturnType};

// Takes the return type (a result) and replaces the error type with &str
fn replace_return_type(r: &ReturnType) -> Result<ReturnType, &'static str> {
    if let syn::ReturnType::Type(_, t) = r {
        if let syn::Type::Path(p) = t.as_ref() {
            let p = &p.path;
            let pairs = p.segments.pairs();
            let value = &pairs.last().unwrap().value().arguments;
            if let PathArguments::AngleBracketed(generics) = value {
                let ok_type = generics.args.first().unwrap();

                return syn::parse::<ReturnType>(quote! {-> Result<#ok_type, &'static str>}.into())
                    .map_err(|_| "Failed to generate return type");
            }
        }
    }
    Err("Could not replace return type")
}

/// Retry the function this macro is attached to if it fails
#[proc_macro_attribute]
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = syn::parse(item).unwrap();
    let retries: syn::Expr = syn::parse(attr).unwrap();

    // Get the function signature and replace the error type with a &str
    let mut sig = input.sig.clone();
    let rt = replace_return_type(&sig.output).unwrap();
    sig.output = rt;

    // the original function needs to be renamed
    let mut original_fun = input.clone();
    original_fun.sig.ident = format_ident!("_{}", sig.ident);
    let original_fun_ident = original_fun.sig.ident.clone();

    let tokens = quote! {
        #original_fun

        #sig {
            let mut tries = 0;
            while tries <= #retries {
                if let Ok(v) = #original_fun_ident() {
                    return Ok(v);
                }

                tries += 1;
            }

            return Err("Exceeded number of tries")
        }
    };
    tokens.into()
}
