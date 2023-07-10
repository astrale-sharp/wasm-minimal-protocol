//! Define conversions from `wasmparser` to `wasm_encoder`

use wasm_encoder as encoder;
use wasmparser as parser;

pub trait ParserToEncoder {
    type EncoderType;
    fn convert(self) -> Self::EncoderType;
}

impl ParserToEncoder for parser::RefType {
    type EncoderType = encoder::RefType;

    fn convert(self) -> Self::EncoderType {
        encoder::RefType {
            nullable: self.is_nullable(),
            heap_type: self.heap_type().convert(),
        }
    }
}

impl ParserToEncoder for parser::HeapType {
    type EncoderType = encoder::HeapType;

    fn convert(self) -> Self::EncoderType {
        match self {
            parser::HeapType::Indexed(idx) => encoder::HeapType::Indexed(idx),
            parser::HeapType::Func => encoder::HeapType::Func,
            parser::HeapType::Extern => encoder::HeapType::Extern,
            parser::HeapType::Any => encoder::HeapType::Any,
            parser::HeapType::None => encoder::HeapType::None,
            parser::HeapType::NoExtern => encoder::HeapType::NoExtern,
            parser::HeapType::NoFunc => encoder::HeapType::NoFunc,
            parser::HeapType::Eq => encoder::HeapType::Eq,
            parser::HeapType::Struct => encoder::HeapType::Struct,
            parser::HeapType::Array => encoder::HeapType::Array,
            parser::HeapType::I31 => encoder::HeapType::I31,
        }
    }
}

impl ParserToEncoder for parser::ValType {
    type EncoderType = encoder::ValType;

    fn convert(self) -> Self::EncoderType {
        match self {
            wasmparser::ValType::I32 => wasm_encoder::ValType::I32,
            wasmparser::ValType::I64 => wasm_encoder::ValType::I64,
            wasmparser::ValType::F32 => wasm_encoder::ValType::F32,
            wasmparser::ValType::F64 => wasm_encoder::ValType::F64,
            wasmparser::ValType::V128 => wasm_encoder::ValType::V128,
            wasmparser::ValType::Ref(r) => wasm_encoder::ValType::Ref(r.convert()),
        }
    }
}

impl ParserToEncoder for parser::TableType {
    type EncoderType = encoder::TableType;

    fn convert(self) -> Self::EncoderType {
        encoder::TableType {
            element_type: self.element_type.convert(),
            minimum: self.initial,
            maximum: self.maximum,
        }
    }
}

impl ParserToEncoder for parser::MemoryType {
    type EncoderType = encoder::MemoryType;

    fn convert(self) -> Self::EncoderType {
        encoder::MemoryType {
            minimum: self.initial,
            maximum: self.maximum,
            memory64: self.memory64,
            shared: self.shared,
        }
    }
}

impl ParserToEncoder for parser::GlobalType {
    type EncoderType = encoder::GlobalType;

    fn convert(self) -> Self::EncoderType {
        encoder::GlobalType {
            val_type: self.content_type.convert(),
            mutable: self.mutable,
        }
    }
}

impl ParserToEncoder for parser::TagType {
    type EncoderType = encoder::TagType;

    fn convert(self) -> Self::EncoderType {
        encoder::TagType {
            kind: encoder::TagKind::Exception,
            func_type_idx: self.func_type_idx,
        }
    }
}

impl ParserToEncoder for parser::TypeRef {
    type EncoderType = encoder::EntityType;

    fn convert(self) -> Self::EncoderType {
        match self {
            parser::TypeRef::Func(idx) => wasm_encoder::EntityType::Function(idx),
            parser::TypeRef::Table(table_type) => {
                wasm_encoder::EntityType::Table(table_type.convert())
            }
            parser::TypeRef::Memory(memory) => wasm_encoder::EntityType::Memory(memory.convert()),
            parser::TypeRef::Global(global) => wasm_encoder::EntityType::Global(global.convert()),
            parser::TypeRef::Tag(tag) => wasm_encoder::EntityType::Tag(tag.convert()),
        }
    }
}
