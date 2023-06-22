/// Minimal protocol for sending/receiving string from and to wasm
/// if you define a function accepting n &str,
/// it will be exposed as a function accepting n integers.
///
/// The last integer will be ignored, the rest will be used to split a
/// concatenated string of the args sent by the host
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use venial::*;

#[proc_macro]
/// Must be called once.
pub fn initiate_protocol(_: TokenStream) -> TokenStream {
    quote!(
        thread_local! {
            #[no_mangle]
            pub static __RESULT: ::std::cell::Cell<Vec<u8>> = ::std::cell::Cell::new(Vec::new());
        }

        #[no_mangle]
        #[export_name = "wasm_minimal_protocol::get_storage_pointer"]
        pub extern "C" fn __wasm_minimal_protocol_internal_function_get_storage_pointer() -> *mut u8
        {
            __RESULT.with(|result| {
                let mut temp = result.replace(Vec::new());
                let ptr = temp.as_mut_ptr();
                result.replace(temp);
                ptr
            })
        }

        #[no_mangle]
        #[export_name = "wasm_minimal_protocol::allocate_storage"]
        pub extern "C" fn __wasm_minimal_protocol_internal_function_allocate_storage(
            length: usize,
        ) {
            __RESULT.with(|x| x.replace(vec![0; length]));
        }

        #[no_mangle]
        #[export_name = "wasm_minimal_protocol::get_storage_len"]
        pub extern "C" fn __wasm_minimal_protocol_internal_function_get_storage_len() -> usize {
            __RESULT.with(|result| {
                let temp = result.replace(Vec::new());
                let len = temp.len();
                result.replace(temp);
                len
            })
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn wasm_func(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    let decl = parse_declaration(item).expect("invalid declaration");
    let func = decl
        .as_function()
        .expect("wasm function proc macro can only be applied to a function")
        .clone();
    let Function {
        name,
        body,
        params,
        vis_marker,
        ..
    } = func;
    //TODO
    match func.return_ty {
        Some(ty) if ty.to_token_stream().to_string() != "String" => panic!(
            "The protocol specifies your function can only return a {}, you tried to return {} ",
            "String",
            ty.to_token_stream()
        ),
        _ => (),
    }
    let p = params
        .items()
        .map(|x| match x {
            FnParam::Receiver(_p) => {
                panic!("args receiving self like {x:?} are not allowed in the protocol")
            }
            FnParam::Typed(p) => {
                let p_to_string = p.ty.to_token_stream().to_string();
                if p.ty.tokens.len() != 2
                    || p.ty.tokens[0].to_string() != "&"
                    || p.ty.tokens[1].to_string() != "str"
                {
                    panic!("only parameter of type &str are allowed, not {p_to_string}")
                }
                p.name.clone()
            }
        })
        .collect::<Vec<_>>();
    let mut get_unsplit_params = quote!(
        let __result = __RESULT.with(|x| {
            x.replace(Vec::new())
        });
        let __unsplit_params = {
            ::std::str::from_utf8(__result.as_slice()) }.unwrap();
    );
    let mut set_args = quote!(
        let start: usize = 0;
    );
    match p.len() {
        0 => get_unsplit_params = quote!(),
        1 => {
            let arg = p.first().unwrap();
            set_args = quote!(
                let #arg: &str = __unsplit_params;
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

    let p_idx = p.iter().map(|name| format_ident!("__{}_idx", name));
    let inner_name = format_ident!("__wasm_minimal_protocol_internal_function_{}", name);
    let export_name = proc_macro2::Literal::string(&name.to_string());
    quote!(
        #vis_marker fn #name(#(#p: &str),*) -> String {
            #[no_mangle]
            #[export_name = #export_name]
            pub extern "C" fn #inner_name( #(#p_idx : usize),* ) {
                #get_unsplit_params
                #set_args
                // get args here

            __RESULT.with(|x| x.replace(#name(#(#p),*).into_bytes()));

            }

            #body
        }
    )
    .into()
}
