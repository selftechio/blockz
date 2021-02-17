//! Blockz derive.

#[cfg(feature = "configuration")]
mod configuration;

#[cfg(feature = "singleton")]
mod singleton;

mod common;
mod paths;

use proc_macro::TokenStream;

use syn::parse_macro_input;
use syn::DeriveInput;
use syn::ItemFn;

/// Derive the Singleton trait.
///
/// This requires that the struct or enum is [Send].
///
/// Required available imports:
/// - [anyhow]
/// - [async_trait]
/// - [blockz]
/// - [once_cell]
/// - [tokio]
///
/// [Send]: https://doc.rust-lang.org/stable/std/marker/trait.Send.html
/// [anyhow]: https://docs.rs/anyhow
/// [async_trait]: https://docs.rs/async_trait
/// [blockz]: https://github.com/selftechio/blockz
/// [once_cell]: https://docs.rs/once_cell
/// [tokio]: https://docs.rs/tokio
#[cfg(feature = "singleton")]
#[proc_macro_derive(Singleton)]
pub fn derive_singleton(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(singleton::derive_singleton(input))
}

#[cfg(feature = "singleton")]
#[proc_macro_attribute]
pub fn singleton_fn(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(singleton::singleton_fn(input))
}

#[cfg(feature = "singleton")]
#[proc_macro_attribute]
pub fn singleton_fn_with_arg(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(singleton::singleton_fn_with_arg(input))
}

#[cfg(feature = "singleton")]
#[proc_macro_attribute]
pub fn singleton_fn_mut(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(singleton::singleton_fn_mut(input))
}

#[cfg(feature = "singleton")]
#[proc_macro_attribute]
pub fn singleton_fn_mut_with_arg(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(singleton::singleton_fn_mut_with_arg(input))
}

/// Derive the Configuration trait.
///
/// All fields shall be loaded from environment variables, at the moment.
///
/// This requires that the struct or enum is [Deserialize].
///
/// Required available imports:
/// - [anyhow]
/// - [async_trait]
/// - [blockz]
/// - [config]
///
/// [Deserialize]: https://docs.rs/serde/1.0.120/serde/trait.Deserialize.html
/// [anyhow]: https://docs.rs/anyhow
/// [async_trait]: https://docs.rs/async_trait
/// [blockz]: https://github.com/selftechio/blockz
/// [config]: https://docs.rs/config
#[proc_macro_derive(Configuration, attributes(config))]
#[cfg(feature = "configuration")]
pub fn derive_configuration(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(configuration::derive_configuration(input))
}
