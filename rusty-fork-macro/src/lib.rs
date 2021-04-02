//-
// Copyright 2018
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(missing_docs, unsafe_code)]

//! Proc-macro crate of `rusty-fork`.

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{AttributeArgs, Error, ItemFn, Lit, Meta, NestedMeta};

/// Run Rust tests in subprocesses.
///
/// The basic usage is to simply put this macro around your test functions.
///
/// ```
/// # /*
/// #[cfg(test)]
/// # */
/// mod test {
///     use rusty_fork::fork_test;
///
///     # /*
///     #[fork_test]
///     # */
///     # pub
///     fn my_test() {
///         assert_eq!(2, 1 + 1);
///     }
///
///     // more tests...
/// }
/// #
/// # fn main() { test::my_test(); }
/// ```
///
/// Each test will be run in its own process. If the subprocess exits
/// unsuccessfully for any reason, including due to signals, the test fails.
///
/// It is also possible to specify a timeout which is applied to all tests in
/// the block, like so:
///
/// ```
/// use rusty_fork::fork_test;
///
/// # /*
/// #[fork_test(timeout_ms = 1000)]
/// # */
/// fn my_test() {
///     do_some_expensive_computation();
/// }
/// # fn do_some_expensive_computation() { }
/// # fn main() { my_test(); }
/// ```
///
/// If any individual test takes more than the given timeout, the child is
/// terminated and the test panics.
///
/// Using the timeout feature requires the `timeout` feature for this crate to
/// be enabled (which it is by default).
///
/// ```
/// use rusty_fork::fork_test;
///
/// # /*
/// #[fork_test(crate = rusty_fork)]
/// # */
/// fn my_test() {
///     assert_eq!(2, 1 + 1);
/// }
/// # fn main() { my_test(); }
/// ```
///
/// Sometimes the crate dependency might be renamed, in cases like this use the `crate` attribute
/// to pass the new name to rusty-fork.
#[proc_macro_attribute]
pub fn fork_test(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as AttributeArgs);

    // defaults
    let mut crate_name = quote::quote! { rusty_fork };
    let mut timeout = quote::quote! {};

    // may be changed by the user
    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(name_value)) = arg {
            if let Some(ident) = name_value.path.get_ident() {
                match ident.to_string().as_str() {
                    "timeout_ms" => {
                        if let Lit::Int(int) = name_value.lit {
                            timeout = quote::quote! { #![rusty_fork(timeout_ms = #int)] }
                        }
                    }
                    "crate" => {
                        if let Lit::Str(str) = name_value.lit {
                            crate_name = str.to_token_stream();
                        }
                    }
                    // we don't support using invalid attributes
                    attribute => {
                        return Error::new(
                            ident.span(),
                            format!(
                                "`{}` is not a valid attribute for `#[fork_test]`",
                                attribute
                            ),
                        )
                        .to_compile_error()
                        .into()
                    }
                }
            }
        }
    }

    let item = syn::parse_macro_input!(item as ItemFn);

    let fn_attrs = item.attrs;
    let fn_vis = item.vis;
    let fn_sig = item.sig;
    let fn_body = item.block;

    // the default is that we add the `#[test]` for the use
    let mut test = quote::quote! { #[test] };

    // we should still support a use case where the user adds it himself
    for attr in &fn_attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "test" {
                test = quote::quote! {};
            }
        }
    }

    // we don't support async functions, whatever library the user uses to support this, should
    // process first
    if let Some(asyncness) = fn_sig.asyncness {
        return Error::new(
            asyncness.span,
            "put `#[fork_test]` after the macro that enables `async` support",
        )
        .to_compile_error()
        .into();
    }

    (quote::quote! {
        ::#crate_name::rusty_fork_test! {
            #timeout

            #test
            #(#fn_attrs)*
            #fn_vis #fn_sig #fn_body
        }
    })
    .into()
}

#[cfg(test)]
mod test {
    use rusty_fork::fork_test;
    use std::io::Result;

    #[fork_test]
    fn trivials() {}

    #[fork_test]
    #[should_panic]
    fn panicking_child() {
        panic!("just testing a panic, nothing to see here");
    }

    #[fork_test]
    #[should_panic]
    fn aborting_child() {
        ::std::process::abort();
    }

    #[fork_test]
    fn trivial_result() -> Result<()> {
        Ok(())
    }

    #[fork_test]
    #[should_panic]
    fn panicking_child_result() -> Result<()> {
        panic!("just testing a panic, nothing to see here");
    }

    #[fork_test]
    #[should_panic]
    fn aborting_child_result() -> Result<()> {
        ::std::process::abort();
    }

    #[fork_test(timeout_ms = 1000)]
    fn timeout_passes() {}

    #[fork_test(timeout_ms = 1000)]
    #[should_panic]
    fn timeout_fails() {
        println!("hello from child");
        ::std::thread::sleep(::std::time::Duration::from_millis(10000));
        println!("goodbye from child");
    }

    #[tokio::test]
    #[fork_test]
    async fn async_test() {
        tokio::task::spawn(async {
            println!("hello from child");
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    #[fork_test]
    async fn async_return_test() -> std::result::Result<(), tokio::task::JoinError> {
        tokio::task::spawn(async {
            println!("hello from child");
        })
        .await
    }
}
