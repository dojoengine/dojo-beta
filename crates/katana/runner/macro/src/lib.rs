//! Implementation of the proc macro in this module is highly adapted from `tokio-macros` crate.
//! `tokio-macro`: https://docs.rs/tokio-macros/2.4.0/tokio_macros/

pub(crate) mod config;
mod entry;
pub(crate) mod parse;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::test(args.into(), input.into()).into()
}
