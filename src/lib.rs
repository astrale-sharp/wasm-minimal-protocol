/// Minimal protocol for sending/receiving string from and to wasm
/// if you define a function accepting n strings,
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
        #[no_mangle]
        static mut RESULT: Vec<u8> = Vec::new();

        #[no_mangle]
        pub fn read_at(at: i32) -> u8 {
            unsafe { RESULT[at as usize] as _ }
        }

        #[no_mangle]
        pub fn get_len() -> usize {
            unsafe { RESULT.len() as _ }
        }

        #[no_mangle]
        pub fn clear() {
            unsafe { RESULT.clear() }
        }

        #[no_mangle]
        pub fn push(chunk: u8) {
            unsafe { RESULT.push(chunk) }
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
        name, body, params, ..
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
                if p.ty.to_token_stream().to_string() != "String" {
                    panic!("only parameter of type string are allowed, not {:?}", p.ty)
                }
                p.name.clone()
            }
        })
        .collect::<Vec<_>>();
    let mut get_unsplit_params = quote!(
        let unsplit_params = unsafe {String::from_utf8(RESULT.clone())}.unwrap();
    );
    let mut set_args = quote!(
        let start: usize = 0;
    );
    match p.len() {
        0 => {
            get_unsplit_params = quote!();
        }
        1 => {
            let arg = p.first().unwrap();
            set_args = quote!(
                let #arg = unsplit_params;
            )
        }
        2.. => {
            // ignore last arg, rest used to split unsplit_param
            let args = &p;
            let mut args_idx = p
                .iter()
                .map(|name| format_ident!("{}_idx", &name))
                .collect::<Vec<_>>();
            args_idx.pop();
            let mut sets = vec![];
            for (idx, arg_idx) in args_idx.iter().enumerate() {
                let arg_name = &args[idx];
                sets.push(quote!(
                    end = #arg_idx as _;
                    let #arg_name = &unsplit_params[start..end];
                    start = end;
                ))
            }
            let last = args.last().unwrap();
            sets.push(quote!(
                let #last =  &unsplit_params[end..];
            ));
            set_args = quote!(
                let mut start : usize = 0;
                let mut end : usize = 0;
                #(
                    #sets
                )*
            );
        }
        _ => unreachable!(),
    }

    let p = p.iter().map(|name| format_ident!("{}_idx", name));
    quote!(
        #[no_mangle]
        #[allow(unused_variables)]
        pub fn #name( #(#p : u32),* ) {
            #get_unsplit_params
            #set_args
            // get args here
            unsafe {
                RESULT = {
                    #body
                }.as_bytes().to_vec();
            }
        }
    )
    .into()
}
