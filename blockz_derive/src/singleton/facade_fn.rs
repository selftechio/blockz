//! Facade fn factory.

use crate::factory::Factory;
use crate::paths;

use super::singleton_fns::SingletonFnArgs;
use super::singleton_fns::SingletonFnType;

use proc_macro2::Ident;
use proc_macro2::TokenStream;

use quote::quote;

use syn::parse::Parser;
use syn::Attribute;
use syn::Block;
use syn::Error;
use syn::FnArg;
use syn::ItemFn;
use syn::Result;

/// Factory that builds the implementation of singleton fns.
pub(super) struct FacadeFnFactory<'f> {
    base: &'f ItemFn,
    fn_type: &'f SingletonFnType<'f>,
    impl_fn: &'f ItemFn,
}

impl<'f> FacadeFnFactory<'f> {
    /// Create a new FacadeFnFactory.
    pub fn new(base: &'f ItemFn, fn_type: &'f SingletonFnType, impl_fn: &'f ItemFn) -> Self {
        Self {
            base,
            fn_type,
            impl_fn,
        }
    }

    /// Remove the receiver of the target function.
    ///
    /// Panic if the first function input is not a receiver.
    fn remove_fn_receiver(target: &mut ItemFn) -> Result<()> {
        let first_arg = if let Some(val) = target.sig.inputs.first() {
            val
        } else {
            return Err(Error::new(
                target.sig.ident.span(),
                format!(
                    "{}: {}: {} {} {}",
                    "facade fn factory",
                    "remove fn receiver",
                    "target function",
                    target.sig.ident,
                    "should have had a receiver",
                ),
            ));
        };
        if let FnArg::Typed(recv) = first_arg {
            return Err(Error::new(
                target.sig.ident.span(),
                format!(
                    "{}: {}: {} {} {}, {}: {:?}",
                    "facade fn factory",
                    "remove fn receiver",
                    "target function",
                    target.sig.ident,
                    "must have either a &self or &mut self receiver as first input",
                    "found first input",
                    recv.ty,
                ),
            ));
        }
        target.sig.inputs = target
            .sig
            .inputs
            .iter()
            .cloned()
            .filter(|arg| !matches!(arg, FnArg::Receiver(_)))
            .collect();
        Ok(())
    }

    /// Replace the block of a function.
    fn replace_fn_block(target: &mut ItemFn, block: TokenStream) -> Result<()> {
        let block: Block = match syn::parse2(quote! {
            {
                #block
            }
        }) {
            Ok(val) => val,
            Err(e) => {
                return Err(Error::new(
                    target.sig.ident.span(),
                    format!(
                        "{}: {}: {}: {}: {}",
                        "facade fn factory",
                        "replace fn block",
                        "failed to parse replacement block",
                        e,
                        block,
                    ),
                ));
            }
        };
        target.block = Box::new(block);
        Ok(())
    }

    /// Adds an #[inline(always)] to the target function.
    fn add_inline_always_attr(target: &mut ItemFn) -> Result<()> {
        let parser = Attribute::parse_outer;
        let parsed_attrs = parser.parse2(quote! {#[inline(always)]})?;
        if parsed_attrs.len() != 1 {
            panic!(
                "{}: {}: {}: {}",
                "facade fn factory",
                "add inline always attr",
                "expected to parse a single attribute",
                "#[inline(always)]"
            );
        }
        let attr_inline = parsed_attrs.into_iter().take(1).next().unwrap();
        target.attrs.push(attr_inline);
        Ok(())
    }

    /// Builds a Singleton::use_singleton.
    fn build_use_singleton_stmt(fn_ident: &Ident) -> TokenStream {
        quote! {
            Self::use_singleton(Self::#fn_ident).await
        }
    }

    /// Builds a Singleton::use_singleton_with_arg.
    fn build_use_singleton_with_arg_stmt(fn_ident: &Ident, arg: &SingletonFnArgs) -> TokenStream {
        let arg = arg.build_impl_fn_call_arg();
        quote! {
            Self::use_singleton_with_arg(Self::#fn_ident, #arg).await
        }
    }

    /// Builds a Singleton::use_mut_singleton.
    fn build_use_mut_singleton_stmt(fn_ident: &Ident) -> TokenStream {
        quote! {
            Self::use_mut_singleton(Self::#fn_ident).await
        }
    }

    /// Builds a Singleton::use_mut_singleton_with_arg.
    fn build_use_mut_singleton_with_arg_stmt(
        fn_ident: &Ident,
        arg: &SingletonFnArgs,
    ) -> TokenStream {
        let arg = arg.build_impl_fn_call_arg();
        quote! {
            Self::use_mut_singleton_with_arg(Self::#fn_ident, #arg).await
        }
    }

    /// Build the facade fn implementation.
    fn build_facade_impl(&self, target: &mut ItemFn) -> Result<()> {
        // get the path to the blockz crate
        let blockz = paths::blockz_path();
        // get the ident of the impl fn
        let impl_fn_ident = &self.impl_fn.sig.ident;
        // build the singleton use statement
        let stmt = match self.fn_type {
            SingletonFnType::NonMut => Self::build_use_singleton_stmt(impl_fn_ident),
            SingletonFnType::NonMutWithArg(arg) => {
                Self::build_use_singleton_with_arg_stmt(impl_fn_ident, arg)
            }
            SingletonFnType::Mut => Self::build_use_mut_singleton_stmt(impl_fn_ident),
            SingletonFnType::MutWithArg(arg) => {
                Self::build_use_mut_singleton_with_arg_stmt(impl_fn_ident, arg)
            }
        };
        // replace the block with the new impl
        Self::replace_fn_block(
            target,
            quote! {
                #[allow(unused_imports)]
                use #blockz::singleton::Singleton;
                #stmt
            },
        )
    }
}

impl<'f> Factory for FacadeFnFactory<'f> {
    type Product = syn::Result<ItemFn>;

    /// Build the facade fn.
    fn build(self) -> Self::Product {
        // create the working copy
        let mut facade_fn = self.base.clone();
        // remove the receiver
        Self::remove_fn_receiver(&mut facade_fn)?;
        // add #[inline(always)] to the function
        Self::add_inline_always_attr(&mut facade_fn)?;
        // build the facade implementation
        self.build_facade_impl(&mut facade_fn)?;
        // return the function
        Ok(facade_fn)
    }
}
