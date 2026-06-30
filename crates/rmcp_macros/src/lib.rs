extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn tool_handler(_attr: TokenStream, item: TokenStream) -> TokenStream { item }

#[proc_macro_attribute]
pub fn tool_router(_attr: TokenStream, item: TokenStream) -> TokenStream { item }

#[proc_macro_attribute]
pub fn tool(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
