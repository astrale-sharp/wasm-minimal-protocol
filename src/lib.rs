use proc_macro::TokenStream;

use quote::{format_ident, quote, ToTokens};
use venial::*;

#[proc_macro]
pub fn declare_protocol(_: TokenStream) -> TokenStream {
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
    let mut p = params
        .items()
        .map(|x| match x {
            FnParam::Receiver(p) => {
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
    let mut get_unsplit_params = quote!();
    let mut set_args = quote!(
        let start: usize = 0;
    );
    // create a func with p.len - 1 args
    if p.len() > 1 {
        let last = p.pop().unwrap();
        get_unsplit_params = quote!(
            let unsplit_params = unsafe {String::from_utf8(RESULT.clone())}.unwrap();
        );
        for (idx, param_name) in p.iter().cloned().enumerate() {
            let param_name_idx = format_ident!("{}_idx", &param_name);
            set_args = quote!(
                #set_args
                let end : usize= #param_name_idx as _;
                let #param_name = &unsplit_params[start..end]; // slice it according to index and the index received arg1 etc
                let start : usize = end;
            )
        }
        set_args = quote!(
            #set_args
            let #last = &unsplit_params[end..];
        )

        // if len == 1 should just get the arg
    } else if p.len() == 1 {
        let last = p.pop().unwrap();
        set_args = quote!(
            let #last = unsafe {String::from_utf8(RESULT.clone())}.unwrap();
        );
    }
    let p = p.iter().map(|name|format_ident!("{}_idx", name));
    quote!(
        #[no_mangle]
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
