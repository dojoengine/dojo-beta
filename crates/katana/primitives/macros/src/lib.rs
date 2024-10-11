use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use starknet::core::types::Felt;
use syn::{parse::Parser, spanned::Spanned, Expr, ExprLit, Ident, Lit};

/// Defines a compile-time constant for a contract address. If the input is a string literal, it will
/// be parsed as a decimal or hexadecimal number. Otherwise, the input will be evaluated as an expression.
///
/// # Examples
///
/// ```rust
/// // Will be evaluated as a constant expression
/// const ADDRESS: Address = address!("0x12345");
/// // or as a runtime expression
/// let address = address!(value);
/// ```
#[proc_macro]
pub fn address(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    inner(input.into()).into()
}

type AttributeArgs = syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>;

fn inner(input: TokenStream) -> TokenStream {
    let args = AttributeArgs::parse_terminated.parse2(input);

    match args {
        Ok(args) => {
            let mut iter = args.iter();
            let arg1 = iter.next();
            let arg2 = iter.next();

            match (arg1, arg2) {
                (Some(value), None) => {
                    let path = Ident::new("katana_primitives", args.span()).into_token_stream();
                    parse_value(path, &value)
                }

                (Some(path), Some(value)) => {
                    if let Expr::Path(path) = path {
                        parse_value(path.into_token_stream(), &value)
                    } else {
                        panic!("invalid crate path")
                    }
                }

                _ => {
                    panic!("invalid macro arguments")
                }
            }
        }

        Err(e) => {
            let mut output = TokenStream::new();
            output.extend(e.into_compile_error());
            output
        }
    }
}

fn parse_value(crate_name: TokenStream, value: &Expr) -> TokenStream {
    match value {
        Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) => {
            let str = lit_str.value();

            let felt = if str.starts_with("0x") {
                Felt::from_hex(&str).expect("invalid Felt value")
            } else {
                Felt::from_dec_str(&str).expect("invalid Felt value")
            };

            let raw = felt.to_raw();
            let raw1 = raw[0];
            let raw2 = raw[1];
            let raw3 = raw[2];
            let raw4 = raw[3];

            let expanded = quote! {
                #crate_path::Address::from_raw([#raw1, #raw2, #raw3, #raw4])
            };

            TokenStream::from(expanded)
        }

        expr => TokenStream::from(quote! { #crate_path::Address::from(#expr) }),
    }
}
