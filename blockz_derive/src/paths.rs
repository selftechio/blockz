//! Paths to libraries used by the expansion of blockz proc-macros.

use proc_macro2::TokenStream;

#[cfg(feature = "no_absolute_paths")]
pub fn anyhow_path() -> TokenStream {
    quote! { anyhow }
}

#[cfg(not(feature = "no_absolute_paths"))]
pub fn anyhow_path() -> TokenStream {
    quote! { ::anyhow }
}

#[cfg(feature = "no_absolute_paths")]
pub fn blockz_path() -> TokenStream {
    quote! { blockz }
}

#[cfg(not(feature = "no_absolute_paths"))]
pub fn blockz_path() -> TokenStream {
    quote! { ::blockz }
}

#[cfg(feature = "no_absolute_paths")]
pub fn envy_path() -> TokenStream {
    quote! { envy }
}

#[cfg(not(feature = "no_absolute_paths"))]
pub fn envy_path() -> TokenStream {
    quote! { ::envy }
}

#[cfg(feature = "no_absolute_paths")]
pub fn once_cell_path() -> TokenStream {
    quote! { once_cell }
}

#[cfg(not(feature = "no_absolute_paths"))]
pub fn once_cell_path() -> TokenStream {
    quote! { ::once_cell }
}

#[cfg(feature = "no_absolute_paths")]
pub fn tokio_path() -> TokenStream {
    quote! { tokio }
}

#[cfg(not(feature = "no_absolute_paths"))]
pub fn tokio_path() -> TokenStream {
    quote! { ::tokio }
}

#[cfg(feature = "no_absolute_paths")]
pub fn std_path() -> TokenStream {
    quote! { std }
}

#[cfg(not(feature = "no_absolute_paths"))]
pub fn std_path() -> TokenStream {
    quote! { ::std }
}
