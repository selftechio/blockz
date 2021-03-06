//! Singleton fns utilities.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;

use proc_macro2::Span;
use proc_macro2::TokenStream;

use quote::quote;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Error;
use syn::FnArg;
use syn::Ident;
use syn::Index;
use syn::ItemFn;
use syn::Pat;
use syn::PatType;
use syn::Receiver;
use syn::Type;
use syn::TypeTuple;

/// The name of the tuple argument for singleton fns.
const SINGLETON_FN_TUPLE_ARG_NAME: &str = "args";

/// Errors produced when attempting to build singleton fns.
enum SingletonFnError<'e> {
    FnHasNoInputs(&'e ItemFn),
    FnReceiverNotRef(&'e Receiver),
    FnArgNotReceiver(&'e PatType),
    FnArgNotTyped(&'e FnArg),
    PatTypeNoIdent(&'e PatType),
}

/// SingletonFn type.
pub(super) enum SingletonFnType<'f> {
    NonMut,
    NonMutWithArg(SingletonFnArgs<'f>),
    Mut,
    MutWithArg(SingletonFnArgs<'f>),
}

/// Arguments for a SingletonFn.
pub(super) enum SingletonFnArgs<'f> {
    /// Single argument.
    ///
    /// This is passed as-is to the singleton function.
    Single {
        /// The identifier of the function argument.
        arg_ident: &'f Ident,
        /// The type of the function argument.
        arg_type: &'f Type,
    },
    /// Multiple arguments.
    ///
    /// These need to be converted to a single argument as a tuple.
    Multiple {
        /// The idents of the original function arguments.
        arg_idents: Vec<&'f Ident>,
        /// The ident of the tuple function argument.
        tuple_ident: Ident,
        /// The type of the tuple function argument.
        ///
        /// This is just a combination of the original types of the tuple function argument.
        tuple_type: TypeTuple,
    },
}

impl<'f> SingletonFnArgs<'f> {
    /// Build the argument used for a singleton call by the facade fn.
    ///
    /// This will either just return the ident of the arg, if there is only
    /// one, or it will return the code needed to build a tuple arg.
    ///
    /// The call arg will be used for:
    /// Singleton::use_singleton_with_arg
    /// or
    /// Singleton::use_mut_singleton_with_arg
    pub fn build_impl_fn_call_arg(&self) -> TokenStream {
        match self {
            SingletonFnArgs::Single { arg_ident, .. } => {
                quote! {#arg_ident}
            }
            SingletonFnArgs::Multiple { arg_idents, .. } => {
                quote! { ( #(#arg_idents),*) }
            }
        }
    }

    /// Build the fn input that will be accepted by the impl fn.
    ///
    /// This builds the `arg: (i64, u64, ...)` in the function signature.
    pub fn build_impl_fn_sig_arg(&self) -> syn::Result<FnArg> {
        match self {
            SingletonFnArgs::Single {
                arg_ident,
                arg_type,
            } => syn::parse2(quote! { #arg_ident: #arg_type }),
            SingletonFnArgs::Multiple {
                tuple_ident,
                tuple_type,
                ..
            } => syn::parse2(quote! { #tuple_ident: #tuple_type }),
        }
    }

    pub fn build_impl_fn_replacement_legend(&self) -> Option<HashMap<String, TokenStream>> {
        match self {
            SingletonFnArgs::Single { .. } => None,
            SingletonFnArgs::Multiple {
                arg_idents,
                tuple_ident,
                ..
            } => {
                let replacement_legend = arg_idents
                    .iter()
                    .enumerate()
                    .map(|(index, arg)| {
                        let index = Index::from(index);
                        (
                            // the name of the argument
                            format!("{}", quote! {#arg}),
                            // the tuple element replacement
                            quote! { #tuple_ident.#index },
                        )
                    })
                    .collect::<HashMap<String, TokenStream>>();
                Some(replacement_legend)
            }
        }
    }
}

/// Get a SingletonFnType from a function.
impl<'f> TryFrom<&'f ItemFn> for SingletonFnType<'f> {
    type Error = syn::Error;
    fn try_from(base: &'f ItemFn) -> syn::Result<Self> {
        // a function must have at least one input (the receiver) to be a
        // singleton function
        if base.sig.inputs.is_empty() {
            return Err(SingletonFnError::FnHasNoInputs(base).into());
        }

        // get the receiver
        // return an error if the receiver is not either self, &self or
        // &mut self
        // safe since we have already checked that there is a non-zero number of inputs
        let receiver = fn_arg_as_receiver(base.sig.inputs.first().unwrap())?;

        // return an error if the receiver is not a reference
        if receiver.reference.is_none() {
            return Err(SingletonFnError::FnReceiverNotRef(receiver).into());
        }

        // check whether the function has other args or not
        let args = {
            if base.sig.inputs.len() == 1 {
                None
            } else {
                Some(
                    base.sig
                        .inputs
                        .iter()
                        .filter_map(|arg| {
                            if let FnArg::Receiver(_) = arg {
                                None
                            } else {
                                Some(fn_arg_as_typed(arg).unwrap())
                            }
                        })
                        .collect::<Vec<&PatType>>(),
                )
            }
        };

        // return the singleton fn type
        if receiver.mutability.is_none() {
            if let Some(value) = args {
                Ok(Self::NonMutWithArg(SingletonFnArgs::try_from(
                    value.as_slice(),
                )?))
            } else {
                Ok(Self::NonMut)
            }
        } else if let Some(value) = args {
            Ok(Self::MutWithArg(SingletonFnArgs::try_from(
                value.as_slice(),
            )?))
        } else {
            Ok(Self::Mut)
        }
    }
}

/// Get the SingletonFnArgs for a series of typed fn args(which have a PatType inside).
impl<'f> TryFrom<&[&'f PatType]> for SingletonFnArgs<'f> {
    type Error = syn::Error;

    fn try_from(src: &[&'f PatType]) -> syn::Result<Self> {
        if src.is_empty() {
            // this should NOT happen
            panic!("singleton fn arg: attempted to construct from a function that has no inputs")
        } else if src.len() == 1 {
            // the function has a single argument
            let arg = src.first().unwrap();
            Ok(Self::Single {
                arg_ident: pat_type_as_ident(*arg)?,
                arg_type: &*arg.ty,
            })
        } else {
            // the function has multiple arguments
            let arg_idents = {
                let mut arg_idents = Vec::with_capacity(src.len());
                for arg in src {
                    arg_idents.push(pat_type_as_ident(arg)?);
                }
                arg_idents
            };

            // create the ident for the args tuple
            let tuple_ident = Ident::new(SINGLETON_FN_TUPLE_ARG_NAME, Span::call_site());

            // create the tuple arg type
            let tuple_type = {
                let elems = src
                    .iter()
                    .map(|arg| *arg.ty.clone())
                    .collect::<Punctuated<Type, syn::token::Comma>>();
                TypeTuple {
                    paren_token: syn::token::Paren {
                        span: Span::call_site(),
                    },
                    elems,
                }
            };

            Ok(Self::Multiple {
                arg_idents,
                tuple_ident,
                tuple_type,
            })
        }
    }
}

impl<'e> Display for SingletonFnError<'e> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SingletonFnError::*;
        match self {
            FnHasNoInputs(_) => {
                write!(f, "attempted to construct a singleton fn from function that has no inputs (must have at least a reference receiver - &self or &mut self)")
            }
            FnReceiverNotRef(_) => {
                write!(
                    f,
                    "function receiver must be a reference receiver (&self or &mut self)"
                )
            }
            FnArgNotReceiver(arg) => {
                let pat = &arg.pat;
                let ty = &arg.ty;
                write!(f, "first function argument {} (type: {}) must be a reference receiver (&self or &mut self)",
                    quote! { #pat },
                    quote! { #ty },
                )
            }
            FnArgNotTyped(_) => write!(f, "function argument is not typed"),
            PatTypeNoIdent(_) => {
                write!(f, "function argument must have an identifier")
            }
        }
    }
}

impl<'e> Spanned for SingletonFnError<'e> {
    fn span(&self) -> Span {
        use SingletonFnError::*;
        match self {
            FnHasNoInputs(f) => f.sig.ident.span(),
            FnReceiverNotRef(recv) => recv.span(),
            FnArgNotReceiver(arg) => arg.span(),
            FnArgNotTyped(arg) => arg.span(),
            PatTypeNoIdent(pat_ty) => pat_ty.span(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl<'e> Into<Error> for SingletonFnError<'e> {
    fn into(self) -> Error {
        Error::new(self.span(), format!("{}", self))
    }
}

/// Get a fn argument as a receiver.
fn fn_arg_as_receiver(src: &FnArg) -> syn::Result<&Receiver> {
    match src {
        FnArg::Receiver(value) => Ok(value),
        FnArg::Typed(bad) => Err(SingletonFnError::FnArgNotReceiver(bad).into()),
    }
}

/// Get a fn argument as a typed argument.
fn fn_arg_as_typed(src: &FnArg) -> syn::Result<&PatType> {
    if let FnArg::Typed(value) = src {
        Ok(value)
    } else {
        Err(SingletonFnError::FnArgNotTyped(src).into())
    }
}

/// Get a pat type as an ident.
fn pat_type_as_ident(src: &PatType) -> syn::Result<&Ident> {
    if let Pat::Ident(value) = &*src.pat {
        Ok(&value.ident)
    } else {
        Err(SingletonFnError::PatTypeNoIdent(src).into())
    }
}
