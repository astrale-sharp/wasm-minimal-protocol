mod parser_to_encoder;

use self::parser_to_encoder::ParserToEncoder as _;
use wasmparser::{Import, Payload, Type, TypeRef};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    WasmParser(#[from] wasmparser::BinaryReaderError),
}

pub fn stub_wasi_functions(binary: &[u8]) -> Result<Vec<u8>, Error> {
    let parser = wasmparser::Parser::default();
    wasmparser::validate(binary)?;

    let payloads = parser.parse_all(binary).collect::<Result<Vec<_>, _>>()?;

    let mut result = wasm_encoder::Module::new();
    let mut types: Vec<Type> = Vec::new();
    let mut to_stub: Vec<Import> = Vec::new();
    let mut code_section = wasm_encoder::CodeSection::new();
    let mut in_code_section = false;
    let mut after_wasi = 0;
    // let mut before_wasi = 0; // TODO

    for payload in &payloads {
        match payload {
            Payload::TypeSection(type_section) => {
                types = type_section
                    .clone()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;
                let (id, range) = payload.as_section().unwrap();
                result.section(&wasm_encoder::RawSection {
                    id,
                    data: &binary[range],
                });
            }
            Payload::ImportSection(import_section) => {
                let mut imports = wasm_encoder::ImportSection::new();
                let mut after_wasi_count = None;
                for import in import_section.clone() {
                    let import = import?;
                    if import.module == "wasi_snapshot_preview1" {
                        after_wasi_count = Some(0);
                        to_stub.push(import);
                    } else {
                        if let Some(n) = after_wasi_count.as_mut() {
                            *n += 1;
                        } else {
                            // before_wasi += 1;  // TODO
                        }
                        imports.import(import.module, import.name, import.ty.convert());
                    }
                }
                after_wasi = after_wasi_count.unwrap_or(0);
                result.section(&imports);
            }
            Payload::FunctionSection(f) => {
                let mut functions_section = wasm_encoder::FunctionSection::new();
                for f in &to_stub {
                    let TypeRef::Func(ty) = f.ty else { continue };
                    functions_section.function(ty);
                }
                for f in f.clone() {
                    functions_section.function(f?);
                }
                result.section(&functions_section);
            }
            Payload::CodeSectionStart { .. } => {
                // TODO: reorder the 'call' instructions in all other functions !
                if after_wasi > 0 {
                    panic!("this crate cannot handle 'wasi_preview' imports that happen after other imports")
                }
                for f in &to_stub {
                    println!("found {}::{}: stubbing...", f.module, f.name);
                    let TypeRef::Func(ty) = f.ty else { continue };
                    let Type::Func(function_type) = &types[ty as usize] else { continue };
                    let locals = function_type
                        .params()
                        .iter()
                        .map(|t| (1u32, t.convert()))
                        .collect::<Vec<_>>();

                    let mut function = wasm_encoder::Function::new(locals);
                    if function_type.results().is_empty() {
                        function.instruction(&wasm_encoder::Instruction::End);
                    } else {
                        function.instruction(&wasm_encoder::Instruction::I32Const(76));
                        function.instruction(&wasm_encoder::Instruction::End);
                    }
                    code_section.function(&function);
                }
                in_code_section = true;
            }
            Payload::CodeSectionEntry(function_body) => {
                code_section.raw(&binary[function_body.range()]);
            }
            _ => {
                if in_code_section {
                    result.section(&code_section);
                    in_code_section = false;
                }
                if let Some((id, range)) = payload.as_section() {
                    result.section(&wasm_encoder::RawSection {
                        id,
                        data: &binary[range],
                    });
                }
            }
        };
    }
    let result = result.finish();
    wasmparser::validate(&result)?;
    Ok(result)
}
