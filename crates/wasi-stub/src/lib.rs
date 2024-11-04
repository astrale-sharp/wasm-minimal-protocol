use std::collections::{HashMap, HashSet};

use wast::{
    core::{
        Expression, Func, FuncKind, FunctionType, HeapType, InlineExport, Instruction, ItemKind,
        Local, ModuleField, ModuleKind, RefType, TypeUse, ValType, InnerTypeKind,
    },
    token::{Id, Index, NameAnnotation},
    Wat,
};

pub enum FunctionsToStub {
    All,
    Some(HashSet<String>),
}
pub struct ShouldStub {
    pub modules: HashMap<String, FunctionsToStub>,
}
impl Default for ShouldStub {
    fn default() -> Self {
        Self {
            modules: [(String::from("wasi_snapshot_preview1"), FunctionsToStub::All)]
                .into_iter()
                .collect(),
        }
    }
}

enum ImportIndex {
    ToStub(u32),
    Keep(u32),
}

struct ToStub {
    fields_index: usize,
    span: wast::token::Span,
    nb_results: usize,
    ty: TypeUse<'static, FunctionType<'static>>,
    name: Option<NameAnnotation<'static>>,
    id: Option<Id<'static>>,
    locals: Vec<Local<'static>>,
}

impl ShouldStub {
    fn should_stub(&self, module: &str, function: &str) -> bool {
        if let Some(functions) = self.modules.get(module) {
            match functions {
                FunctionsToStub::All => true,
                FunctionsToStub::Some(functions) => functions.contains(function),
            }
        } else {
            false
        }
    }
}

fn static_id(id: Option<Id>) -> Option<Id<'static>> {
    id.map(|id| {
        let mut name = id.name().to_owned();
        name.insert(0, '$');
        let parser = Box::leak(Box::new(
            wast::parser::ParseBuffer::new(name.leak()).unwrap(),
        ));
        wast::parser::parse::<Id>(parser).unwrap()
    })
}
fn static_name_annotation(name: Option<NameAnnotation>) -> Option<NameAnnotation<'static>> {
    name.map(|name| NameAnnotation {
        name: String::from(name.name).leak(),
    })
}

pub fn stub_wasi_functions(
    binary: &[u8],
    should_stub: ShouldStub,
    return_value: u32,
) -> anyhow::Result<Vec<u8>> {
    let wat = wasmprinter::print_bytes(binary)?;
    let parse_buffer = wast::parser::ParseBuffer::new(&wat)?;

    let mut wat: Wat = wast::parser::parse(&parse_buffer)?;
    let module = match &mut wat {
        Wat::Module(m) => m,
        Wat::Component(_) => {
            anyhow::bail!("components are not supported")
        }
    };
    let fields = match &mut module.kind {
        ModuleKind::Text(f) => f,
        ModuleKind::Binary(_) => {
            println!("[WARNING] binary directives are not supported");
            return Ok(binary.to_owned());
        }
    };

    let mut types = Vec::new();
    let mut imports = Vec::new();
    let mut to_stub = Vec::new();
    let mut insert_stubs_index = None;
    let mut new_import_indices = Vec::new();

    for (field_idx, field) in fields.iter_mut().enumerate() {
        match field {
            ModuleField::Type(t) => types.push(t),
            ModuleField::Import(i) => {
                let typ = match &i.item.kind {
                    ItemKind::Func(typ) => typ.index.and_then(|index| match index {
                        Index::Num(index, _) => Some(index as usize),
                        Index::Id(_) => None,
                    }),
                    _ => None,
                };
                let new_index = match typ {
                    Some(type_index) if should_stub.should_stub(i.module, i.field) => {
                        println!("Stubbing function {}::{}", i.module, i.field);
                        let typ = &types[type_index];
                        let ty = TypeUse::new_with_index(Index::Num(type_index as u32, typ.span));
                        let wast::core::TypeDef{kind: InnerTypeKind::Func(func_typ), ..} = &typ.def else {
                            continue;
                        };
                        let id = static_id(i.item.id);
                        let locals: Vec<Local> = func_typ
                            .params
                            .iter()
                            .map(|(id, name, val_type)| Local {
                                id: static_id(*id),
                                name: static_name_annotation(*name),
                                // FIXME: This long match dance is _only_ to make the lifetime of ty 'static. A lot of things have to go through this dance (see the `static_*` function...)
                                // Instead, we should write the new function here, in place, by replacing `field`. This is currently done in the for loop at the veryend of this function.
                                // THEN, at the end of the loop, swap every function in it's right place. No need to do more !
                                ty: match val_type {
                                    ValType::I32 => ValType::I32,
                                    ValType::I64 => ValType::I64,
                                    ValType::F32 => ValType::F32,
                                    ValType::F64 => ValType::F64,
                                    ValType::V128 => ValType::V128,
                                    ValType::Ref(r) => ValType::Ref(RefType {
                                        nullable: r.nullable,
                                        heap: match r.heap {
                                            HeapType::Concrete(index) => {
                                                HeapType::Concrete(match index {
                                                    Index::Num(n, s) => Index::Num(n, s),
                                                    Index::Id(id) => {
                                                        Index::Id(static_id(Some(id)).unwrap())
                                                    }
                                                })
                                            }
                                            HeapType::Abstract { shared, ty } => HeapType::Abstract { shared, ty },
                                        },
                                    }),
                                },
                            })
                            .collect();
                        to_stub.push(ToStub {
                            fields_index: field_idx,
                            span: i.span,
                            nb_results: func_typ.results.len(),
                            ty,
                            name: i.item.name.map(|n| NameAnnotation {
                                name: n.name.to_owned().leak(),
                            }),
                            id,
                            locals,
                        });
                        ImportIndex::ToStub(to_stub.len() as u32 - 1)
                    }
                    _ => {
                        imports.push(i);
                        ImportIndex::Keep(imports.len() as u32 - 1)
                    }
                };
                new_import_indices.push(new_index);
            }
            ModuleField::Func(func) => {
                if insert_stubs_index.is_none() {
                    insert_stubs_index = Some(field_idx);
                }
                match &mut func.kind {
                    FuncKind::Import(f) => {
                        if should_stub.should_stub(f.module, f.field) {
                            println!("[WARNING] Stubbing inline function is not yet supported");
                            println!(
                                "[WARNING] ignoring inline function \"{}\" \"{}\"",
                                f.module, f.field
                            );
                        }
                    }
                    FuncKind::Inline { expression, .. } => {
                        for inst in expression.instrs.as_mut().iter_mut() {
                            match inst {
                                Instruction::RefFunc(Index::Num(index, _))
                                | Instruction::ReturnCall(Index::Num(index, _))
                                | Instruction::Call(Index::Num(index, _)) => {
                                    if let Some(new_index) = new_import_indices.get(*index as usize)
                                    {
                                        *index = match new_index {
                                            ImportIndex::ToStub(idx) => *idx + imports.len() as u32,
                                            ImportIndex::Keep(idx) => *idx,
                                        };
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    drop(imports);
    drop(types);

    let insert_stubs_index = insert_stubs_index
        .expect("This is weird: there are no code sections in this wasm executable !");

    for (
        already_stubbed,
        ToStub {
            fields_index,
            span,
            nb_results,
            ty,
            name,
            id,
            locals,
        },
    ) in to_stub.into_iter().enumerate()
    {
        let instructions = {
            let mut res = Vec::with_capacity(nb_results);
            for _ in 0..nb_results {
                // Weird value, hopefully this makes it easier to track usage of these stubbed functions.
                res.push(Instruction::I32Const(return_value as i32));
            }
            res
        };
        let function = Func {
            span,
            id,
            name,
            // no exports
            exports: InlineExport { names: Vec::new() },
            kind: wast::core::FuncKind::Inline {
                locals: locals.into_boxed_slice(),
                expression: Expression {
                    instrs: instructions.into_boxed_slice(),
                    branch_hints: Box::new([]),
                    instr_spans: None,
                },
            },
            ty,
        };
        fields.insert(insert_stubs_index, ModuleField::Func(function));
        fields.remove(fields_index - already_stubbed);
    }

    Ok(module.encode()?)
}
