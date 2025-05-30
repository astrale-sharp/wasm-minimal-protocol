//! Minimal protocol for sending/receiving messages from and to a wasm host.
//!
//! If you define a function accepting `n` arguments of type `&[u8]`, it will
//! internally be exported as a function accepting `n` integers.
//!
//! # Example
//!
//! ```
//! use wasm_minimal_protocol::wasm_func;
//!
//! #[cfg(target_arch = "wasm32")]
//! wasm_minimal_protocol::initiate_protocol!();
//!
//! #[cfg_attr(target_arch = "wasm32", wasm_func)]
//! fn concatenate(arg1: &[u8], arg2: &[u8]) -> Vec<u8> {
//!     [arg1, arg2].concat()
//! }
//! ```
//!
//! # Allowed types
//!
//! Allowed input types are either `&[u8]` or `&mut [u8]`.
//!
//! Allowed output types are
//! - `Vec<u8>`
//! - `Box<[u8]>`
//! - `&[u8]`
//! - `Result<T, E>`, where `T` is any of the above, and `E` is a type implementing
//!   [`Display`](std::fmt::Display).
//!
//! # Protocol
//!
//! The specification of the low-level protocol can be found in the typst documentation:
//! <https://typst.app/docs/reference/foundations/plugin/#protocol>

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenTree};
use quote::{format_ident, quote, ToTokens};
use venial::*;

/// Macro that sets up the correct imports and traits to be used by [`macro@wasm_func`].
///
/// This macro should be called only once, preferably at the root of the crate. It does
/// not take any arguments.
#[proc_macro]
pub fn initiate_protocol(stream: TokenStream) -> TokenStream {
    if !stream.is_empty() {
        return quote!(
            compile_error!("This macro does not take any arguments");
        )
        .into();
    }
    quote!(
        #[link(wasm_import_module = "typst_env")]
        extern "C" {
            #[link_name = "wasm_minimal_protocol_send_result_to_host"]
            fn __send_result_to_host(ptr: *const u8, len: usize);
            #[link_name = "wasm_minimal_protocol_write_args_to_buffer"]
            fn __write_args_to_buffer(ptr: *mut u8);
        }

        trait __ToResult {
            type Ok: ::core::convert::AsRef<[u8]>;
            type Err: ::core::fmt::Display;
            fn to_result(self) -> ::core::result::Result<Self::Ok, Self::Err>;
        }
        impl __ToResult for Vec<u8> {
            type Ok = Self;
            type Err = ::core::convert::Infallible;
            fn to_result(self) -> ::core::result::Result<Self::Ok, Self::Err> {
                Ok(self)
            }
        }
        impl __ToResult for Box<[u8]> {
            type Ok = Self;
            type Err = ::core::convert::Infallible;
            fn to_result(self) -> ::core::result::Result<Self::Ok, Self::Err> {
                Ok(self)
            }
        }
        impl<'a> __ToResult for &'a [u8] {
            type Ok = Self;
            type Err = ::core::convert::Infallible;
            fn to_result(self) -> ::core::result::Result<Self::Ok, Self::Err> {
                Ok(self)
            }
        }
        impl<T: ::core::convert::AsRef<[u8]>, E: ::core::fmt::Display> __ToResult for ::core::result::Result<T, E> {
            type Ok = T;
            type Err = E;
            fn to_result(self) -> Self {
                self
            }
        }
    )
    .into()
}

/// Wrap the function to be used with the [protocol](https://typst.app/docs/reference/foundations/plugin/#protocol).
///
/// # Arguments
///
/// All the arguments of the function should be `&[u8]`, no lifetime needed.
///
/// # Return type
///
/// The return type of the function should be `Vec<u8>` or `Result<Vec<u8>, E>` where
/// `E: ToString`.
///
/// If the function return `Vec<u8>`, it will be implicitely wrapped in `Ok`.
///
/// # Example
///
/// ```
/// use wasm_minimal_protocol::wasm_func;
///
/// #[cfg(target_arch = "wasm32")]
/// wasm_minimal_protocol::initiate_protocol!();
///
/// #[cfg_attr(target_arch = "wasm32", wasm_func)]
/// fn function_one() -> Vec<u8> {
///     Vec::new()
/// }
///
/// #[cfg_attr(target_arch = "wasm32", wasm_func)]
/// fn function_two(arg1: &[u8], arg2: &[u8]) -> Result<Vec<u8>, i32> {
///     Ok(b"Normal message".to_vec())
/// }
///
/// #[cfg_attr(target_arch = "wasm32", wasm_func)]
/// fn function_three(arg1: &[u8]) -> Result<Vec<u8>, String> {
///     Err(String::from("Error message"))
/// }
/// ```
#[proc_macro_attribute]
pub fn wasm_func(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = proc_macro2::TokenStream::from(item);
    let decl = parse_declaration(item.clone()).expect("invalid declaration");
    let func = match decl.as_function() {
        Some(func) => func.clone(),
        None => {
            let error = venial::Error::new_at_tokens(
                &item,
                "#[wasm_func] can only be applied to a function",
            );
            item.extend(error.to_compile_error());
            return item.into();
        }
    };
    let Function {
        name,
        params,
        vis_marker,
        ..
    } = func.clone();

    let mut error = None;

    let p = params
        .items()
        .filter_map(|x| match x {
            FnParam::Receiver(_p) => {
                let x = x.to_token_stream();
                error = Some(venial::Error::new_at_tokens(
                    &x,
                    format!("the `{x}` argument is not allowed by the protocol"),
                ));
                None
            }
            FnParam::Typed(p) => {
                if !tokens_are_slice(&p.ty.tokens) {
                    let p_to_string = p.ty.to_token_stream();
                    error = Some(venial::Error::new_at_tokens(
                        &p_to_string,
                        format!("only parameters of type `&[u8]` or `&mut [u8]` are allowed, not {p_to_string}"),
                    ));
                    None
                } else {
                    Some(p.name.clone())
                }
            }
        })
        .collect::<Vec<_>>();
    let p_len = p
        .iter()
        .map(|name| format_ident!("__{}_len", name))
        .collect::<Vec<_>>();

    let mut get_unsplit_params = quote!(
        let __total_len = #(#p_len + )* 0;
        let mut __unsplit_params = vec![0u8; __total_len];
        unsafe { __write_args_to_buffer(__unsplit_params.as_mut_ptr()); }
        let __unsplit_params: &mut [u8] = &mut __unsplit_params;
    );
    let mut set_args = quote!(
        let start: usize = 0;
    );
    match p.len() {
        0 => get_unsplit_params = quote!(),
        1 => {
            let arg = p.first().unwrap();
            set_args = quote!(
                let #arg: &mut [u8] = __unsplit_params;
            )
        }
        _ => {
            let mut sets = vec![];
            for (idx, (arg_name, arg_len)) in p
                .iter()
                .zip(p.iter().map(|name| format_ident!("__{}_len", &name)))
                .enumerate()
            {
                if idx == p.len() - 1 {
                    sets.push(quote!(
                        let #arg_name: &mut [u8] = __unsplit_params;
                    ));
                } else {
                    sets.push(quote!(
                        let (#arg_name, __unsplit_params): (&mut [u8], &mut [u8]) =
                            __unsplit_params.split_at_mut(#arg_len);
                    ));
                }
            }
            set_args = quote!(
                #(
                    #sets
                )*
            );
        }
    }

    let inner_name = format_ident!("__wasm_minimal_protocol_internal_function_{}", name);
    let export_name = proc_macro2::Literal::string(&name.to_string());

    let mut result = quote!(#func);
    if let Some(error) = error {
        result.extend(error.to_compile_error());
    } else {
        result.extend(quote!(
            #[export_name = #export_name]
            #vis_marker extern "C" fn #inner_name(#(#p_len: usize),*) -> i32 {
                #get_unsplit_params
                #set_args

                let result = #name(#(#p),*);
                let result = __ToResult::to_result(result);
                let err_vec: Vec<u8>;
                let (message, code) = match result {
                    Ok(ref s) => (s.as_ref(), 0),
                    Err(err) => {
                        err_vec = err.to_string().into_bytes();
                        (err_vec.as_slice(), 1)
                    },
                };
                unsafe { __send_result_to_host(message.as_ptr(), message.len()); }
                code
            }
        ))
    }
    result.into()
}

/// Check that `ty` is either `&[u8]` or `&mut [u8]`.
fn tokens_are_slice(ty: &[TokenTree]) -> bool {
    let is_ampersand = |t: &_| matches!(t, TokenTree::Punct(punct) if punct.as_char() == '&');
    let is_quote = |t: &_| matches!(t, TokenTree::Punct(punct) if punct.as_char() == '\'');
    let is_sym = |t: &_| matches!(t, TokenTree::Ident(_));
    let is_mut = |t: &_| matches!(t, TokenTree::Ident(i) if i == "mut");
    let is_sliceu8 = |t: &_| match t {
        TokenTree::Group(group) => {
            group.delimiter() == Delimiter::Bracket && {
                let mut inner = group.stream().into_iter();
                matches!(
                    inner.next(),
                    Some(proc_macro2::TokenTree::Ident(i)) if i == "u8"
                ) && inner.next().is_none()
            }
        }
        _ => false,
    };
    let mut iter = ty.iter();
    let Some(amp) = iter.next() else { return false };
    if !is_ampersand(amp) {
        return false;
    }
    let Some(mut next) = iter.next() else {
        return false;
    };
    if is_quote(next) {
        // there is a lifetime
        let Some(lft) = iter.next() else { return false };
        if !is_sym(lft) {
            return false;
        }
        match iter.next() {
            Some(t) => next = t,
            None => return false,
        }
    }
    if is_mut(next) {
        // the slice is mutable
        match iter.next() {
            Some(t) => next = t,
            None => return false,
        }
    }
    if !is_sliceu8(next) {
        return false;
    }
    if iter.next().is_some() {
        return false;
    }

    true
}
