//! Minimal protocol for sending/receiving string from and to a wasm host.
//!
//! If you define a function accepting `n` strings (`&str`, not `String`), it will
//! internally be exported as a function accepting `n` integers.
//!
//! # Example
//!
//! ```
//! use wasm_minimal_protocol::wasm_func;
//!
//! wasm_minimal_protocol::initiate_protocol!();
//!
//! #[wasm_func]
//! fn concatenate(arg1: &str, arg2: &str) -> String {
//!     format!("{}{}", arg1, arg2)
//! }
//! ```

/// Documentation-only item, to describe the low-level protocol used by this crate.
///
#[cfg(doc)]
#[doc = include_str!("../protocol.md")]
#[proc_macro]
pub fn protocol(stream: TokenStream) -> TokenStream {}

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use venial::*;

/// Macro that sets up the correct imports and traits to be used by [`macro@wasm_func`].
///
/// This macro should be called only once, preferably at the root of the crate. It does
/// not take any arguments.
#[proc_macro]
pub fn initiate_protocol(stream: TokenStream) -> TokenStream {
    let mut result = quote!(
        #[cfg(not(target_arch = "wasm32"))]
        compile_error!("Error: this protocol may only be used when compiling to wasm architectures");

        #[link(wasm_import_module = "typst_env")]
        extern "C" {
            #[link_name = "wasm_minimal_protocol_send_result_to_host"]
            fn __send_result_to_host(ptr: *const u8, len: usize);
            #[link_name = "wasm_minimal_protocol_write_args_to_buffer"]
            fn __write_args_to_buffer(ptr: *mut u8);
        }

        trait __StringOrResultString {
            type Err;
            fn convert(self) -> ::std::result::Result<String, Self::Err>;
        }
        impl __StringOrResultString for String {
            type Err = String;
            fn convert(self) -> ::std::result::Result<String, <Self as __StringOrResultString>::Err> {
                Ok(self)
            }
        }
        impl<E> __StringOrResultString for ::std::result::Result<String, E> {
            type Err = E;
            fn convert(self) -> ::std::result::Result<String, <Self as __StringOrResultString>::Err> {
                self
            }
        }
    );
    if !stream.is_empty() {
        result.extend(quote!(
            compile_error!("This macro does not take any arguments");
        ));
    }
    result.into()
}

/// Wrap the function to be used with the [protocol!].
///
/// # Arguments
///
/// All the arguments of the function should be `&str`, without lifetimes.
///
/// # Return type
///
/// The return type of the function should be `String` or `Result<String, E>` where
/// `E: ToString`.
///
/// If the function return `String`, it will be implicitely wrapped in `Ok`.
///
/// # Example
///
/// ```
/// use wasm_minimal_protocol::wasm_func;
///
/// wasm_minimal_protocol::initiate_protocol!();
///
/// #[wasm_func]
/// fn function_one() -> String {
///     String::new()
/// }
///
/// #[wasm_func]
/// fn function_two(arg1: &str, arg2: &str) -> Result<String, i32> {
///     Ok(String::from("Normal message"))
/// }
///
/// #[wasm_func]
/// fn function_three(arg1: &str) -> Result<String, String> {
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
                    format!("the {x} argument is not allowed by the protocol"),
                ));
                None
            }
            FnParam::Typed(p) => {
                if p.ty.tokens.len() != 2
                    || p.ty.tokens[0].to_string() != "&"
                    || p.ty.tokens[1].to_string() != "str"
                {
                    let p_to_string = p.ty.to_token_stream();
                    error = Some(venial::Error::new_at_tokens(
                        &p_to_string,
                        format!("only parameter of type &str are allowed, not {p_to_string}"),
                    ));
                    None
                } else {
                    Some(p.name.clone())
                }
            }
        })
        .collect::<Vec<_>>();
    let p_idx = p
        .iter()
        .map(|name| format_ident!("__{}_idx", name))
        .collect::<Vec<_>>();

    let mut get_unsplit_params = quote!(
        let __total_len = #(#p_idx + )* 0;
        let mut __unsplit_params = vec![0u8; __total_len];
        unsafe { __write_args_to_buffer(__unsplit_params.as_mut_ptr()); }
        let __unsplit_params = String::from_utf8(__unsplit_params).unwrap();
    );
    let mut set_args = quote!(
        let start: usize = 0;
    );
    match p.len() {
        0 => get_unsplit_params = quote!(),
        1 => {
            let arg = p.first().unwrap();
            set_args = quote!(
                let #arg: &str = &__unsplit_params;
            )
        }
        _ => {
            // ignore last arg, rest used to split unsplit_param
            let args = &p;
            let mut args_idx = p
                .iter()
                .map(|name| format_ident!("__{}_idx", &name))
                .collect::<Vec<_>>();
            args_idx.pop();
            let mut sets = vec![];
            let mut start = quote!(0usize);
            let mut end = quote!(0usize);
            for (idx, arg_idx) in args_idx.iter().enumerate() {
                end = quote!(#end + #arg_idx);
                let arg_name = &args[idx];
                sets.push(quote!(
                    let #arg_name: &str = &__unsplit_params[#start..#end];
                ));
                start = quote!(#start + #arg_idx)
            }
            let last = args.last().unwrap();
            sets.push(quote!(
                let #last = &__unsplit_params[#end..];
            ));
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
            #vis_marker fn #inner_name(#(#p_idx: usize),*) -> i32 {
                #get_unsplit_params
                #set_args

                let result = __StringOrResultString::convert(#name(#(#p),*));
                let (string, code) = match result {
                    Ok(s) => (s, 0),
                    Err(err) => (err.to_string(), 1),
                };
                unsafe { __send_result_to_host(string.as_ptr(), string.len()); }
                code // indicates everything was successful
            }
        ))
    }
    result.into()
}
