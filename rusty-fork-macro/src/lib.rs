#![deny(missing_docs, unsafe_code)]
#![allow(unused_imports, unused_variables)]

//! Proc-macro crate of `rusty-fork`.

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{AttributeArgs, Error, ItemFn, Lit, Meta, NestedMeta, ReturnType};

/// Run Rust tests in subprocesses.
///
/// The basic usage is to simply put this macro around your test functions.
///
/// ```
/// # /*
/// #[cfg(test)]
/// # */
/// mod test {
///     use rusty_fork::test_fork;
///
///     # /*
///     #[test_fork]
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
/// use rusty_fork::test_fork;
///
/// # /*
/// #[test_fork(timeout_ms = 1000)]
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
#[proc_macro_attribute]
pub fn test_fork(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as AttributeArgs);

    let mut crate_name = quote::quote! { rusty_fork };
    let mut timeout = quote::quote! { 0 };

    for arg in args {
        if let NestedMeta::Meta(meta) = arg {
            if let Meta::NameValue(name_value) = meta {
                if let Some(ident) = name_value.path.get_ident() {
                    match ident.to_string().as_str() {
                        "timeout_ms" => {
                            if let Lit::Int(int) = name_value.lit {
                                timeout = int.to_token_stream();
                            }
                        }
                        "crate" => {
                            if let Lit::Str(str) = name_value.lit {
                                crate_name = str.to_token_stream();
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    let item = syn::parse_macro_input!(item as ItemFn);

    let fn_attrs = &item.attrs;
    let fn_sig = &item.sig;
    let fn_body = &item.block;
    let fn_name = &fn_sig.ident;

    let mut test = quote::quote! { #[test] };

    for attr in fn_attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "test" {
                test = quote::quote! {};
            }
        }
    }

    if let Some(asyncness) = fn_sig.asyncness {
        return Error::new(
            asyncness.span,
            "put `#[test_fork]` last to let other macros process first",
        )
        .to_compile_error()
        .into();
    }

    let body_fn = if let ReturnType::Type(_, ret_ty) = &fn_sig.output {
        quote::quote! {
            fn body_fn() {
                fn body_fn() -> #ret_ty #fn_body
                body_fn().unwrap();
            }
        }
    } else {
        quote::quote! {
            fn body_fn() #fn_body
        }
    };

    (quote::quote! {
        #test
        #(#fn_attrs)*
        fn #fn_name() {
            // Eagerly convert everything to function pointers so that all
            // tests use the same instantiation of `fork`.
            #body_fn
            let body: fn () = body_fn;

            fn supervise_fn(
                child: &mut ::#crate_name::ChildWrapper,
                _file: &mut ::std::fs::File
            ) {
                ::#crate_name::fork_test::supervise_child(child, #timeout)
            }
            let supervise:
                fn (&mut ::#crate_name::ChildWrapper, &mut ::std::fs::File) =
                supervise_fn;
            ::#crate_name::fork(
                ::#crate_name::rusty_fork_test_name!(#fn_name),
                ::#crate_name::rusty_fork_id!(),
                ::#crate_name::fork_test::no_configure_child,
                supervise,
                body
            ).expect("forking test failed")
        }
    })
    .into()
}

#[cfg(test)]
mod test {
    use rusty_fork::test_fork;
    use std::io::Result;

    #[test_fork]
    fn trivials() {}

    #[test_fork]
    #[should_panic]
    fn panicking_child() {
        panic!("just testing a panic, nothing to see here");
    }

    #[test_fork]
    #[should_panic]
    fn aborting_child() {
        ::std::process::abort();
    }

    #[test_fork]
    fn trivial_result() -> Result<()> {
        Ok(())
    }

    #[test_fork]
    #[should_panic]
    fn panicking_child_result() -> Result<()> {
        panic!("just testing a panic, nothing to see here");
    }

    #[test_fork]
    #[should_panic]
    fn aborting_child_result() -> Result<()> {
        ::std::process::abort();
    }

    #[test_fork(timeout_ms = 1000)]
    fn timeout_passes() {}

    #[test_fork(timeout_ms = 1000)]
    #[should_panic]
    fn timeout_fails() {
        println!("hello from child");
        ::std::thread::sleep(::std::time::Duration::from_millis(10000));
        println!("goodbye from child");
    }

    #[tokio::test]
    #[test_fork]
    async fn my_test() {
        assert!(true);
    }
}
