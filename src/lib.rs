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
///
/// Why not a simple static ? why not a thread_local and/or a cell? why no defense against data race ?
///
/// The plugin will never be aware if it is being sent in a thread and hence always single threaded, it's the host that needs to make sure to be Send and Sync.
///
/// It's also wanted that the implementation is simple and easy to read so that it can be adapted to C or C++ easily.
pub fn initiate_protocol(_: TokenStream) -> TokenStream {
    quote!(
        #[link(wasm_import_module = "typst_env")]
        extern "C" {
            #[link_name = "wasm_minimal_protocol_send_result_to_host"]
            fn __send_result_to_host(ptr: *const u8, len: usize);
            #[link_name = "wasm_minimal_protocol_write_args_to_buffer"]
            fn __write_args_to_buffer(ptr: *mut u8);
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
        params,
        vis_marker,
        ..
    } = func.clone();

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
    quote!(
        #func

        #[export_name = #export_name]
        #vis_marker fn #inner_name(#(#p_idx: usize),*) -> usize {
                #get_unsplit_params
                #set_args

                let result = #name(#(#p),*);
                unsafe { __send_result_to_host(result.as_ptr(), result.len()); }
                0 // indicates everything was successful
        }
    )
    .into()
}
