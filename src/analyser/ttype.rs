use std::fmt::Display;

use crate::parser::tree::Literal;

use super::{stack_effect::StackEffect, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	Opaque {
		size: Option<usize>,
		kind: OpaqueTypeKind
	},
	Transparent {
		name: String,
		fields: im::OrdMap<String, Type>,
		/// Whether this type is a sum type/enum (true) or product type/struct (false)
		sum_type: bool,
	},
	Reference {
		to: Box<Type>,
	},
	Generic {
		name: String,
	},
	Function {
		name: String,
		effect: StackEffect
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpaqueTypeKind {
	UnsignedInt,
	SignedInt,
	Float,
	Bool,
	Str,
	Array
}

impl Type {
	pub fn new_int(bits: usize) -> Type {
		Type::Opaque { size: Some(bits / 8), kind: OpaqueTypeKind::SignedInt }
	}

	pub fn new_uint(bits: usize) -> Type {
		Type::Opaque { size: Some(bits / 8), kind: OpaqueTypeKind::UnsignedInt }
	}

	pub fn new_bool() -> Type {
		Type::Opaque { size: Some(1), kind: OpaqueTypeKind::Bool }
	}

	pub fn new_str(len_bytes: usize) -> Type {
		Type::Opaque { size: Some(len_bytes), kind: OpaqueTypeKind::Str }
	}

	pub fn new_strref(len_bytes: usize) -> Type {
		Type::Reference { to: Box::new(Type::new_str(len_bytes)) }
	}

	pub fn new_struct(name: String, fields: &im::OrdMap<String, Type>) -> Type {
		Type::Transparent { name, fields: fields.clone(), sum_type: false }
	}

	pub fn new_enum(name: String, fields: &im::OrdMap<String, Type>) -> Type {
		Type::Transparent { name, fields: fields.clone(), sum_type: true }
	}

	pub fn from_lit(lit: &Literal) -> Type {
		match lit {
			Literal::U128(_) => Type::new_uint(128),
			Literal::U64(_) => Type::new_uint(64),
			Literal::U32(_) => Type::new_uint(32),
			Literal::U16(_) => Type::new_uint(16),
			Literal::U8(_) => Type::new_uint(8),
			Literal::I128(_) => Type::new_int(128),
			Literal::I64(_) => Type::new_int(64),
			Literal::I32(_) => Type::new_int(32),
			Literal::I16(_) => Type::new_int(16),
			Literal::I8(_) => Type::new_int(8),
			Literal::F64(_) => todo!(),
			Literal::F32(_) => todo!(),
			Literal::Bool(_) => Type::new_bool(),
			Literal::String(val) => Type::new_strref(val.len()),
			Literal::FnPtr(val) => todo!(),
		}
	}

	pub fn from_name(name: impl AsRef<str>) -> Option<Type> {
		let name = name.as_ref();

		if name.starts_with("&") {
			return Some(Type::Reference { to: Box::new(Type::from_name(&name[1..])?) });
		}

		match name {
			"u128" => Some(Type::new_uint(128)),
			"u64" => Some(Type::new_uint(64)),
			"u32" => Some(Type::new_uint(32)),
			"u16" => Some(Type::new_uint(16)),
			"u8" => Some(Type::new_uint(8)),
			"i128" => Some(Type::new_int(128)),
			"i64" => Some(Type::new_int(64)),
			"i32" => Some(Type::new_int(32)),
			"i16" => Some(Type::new_int(16)),
			"i8" => Some(Type::new_int(8)),
			"bool" => Some(Type::new_bool()),
			_ => None
		}
	}

	pub fn name(&self) -> String {
		match self {
			Type::Opaque { size, kind } => {
				match size {
					Some(size) => {
						match kind {
							OpaqueTypeKind::UnsignedInt => format!("u{}", size * 8),
							OpaqueTypeKind::SignedInt => format!("i{}", size * 8),
							OpaqueTypeKind::Float => format!("f{}", size * 8),
							OpaqueTypeKind::Bool => "bool".to_string(),
							OpaqueTypeKind::Str => todo!(),
							OpaqueTypeKind::Array => todo!(),
						}
					}
					None => {
						todo!()
					}
				}
			},
			Type::Transparent { name, fields, sum_type } => todo!(),
			Type::Reference { to } => todo!(),
			Type::Generic { name } => todo!(),
			Type::Function { name, effect } => todo!(),
		}
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", {
			match self {
				Self::Opaque { size, kind } => {
					if let Some(size) = size {
						match kind {
							OpaqueTypeKind::UnsignedInt => format!("u{}", size * 8),
							OpaqueTypeKind::SignedInt => format!("i{}", size * 8),
							OpaqueTypeKind::Float => format!("f{}", size * 8),
							OpaqueTypeKind::Bool => format!("bool"),
							OpaqueTypeKind::Str => format!("str(byte_len: {})", size),
							OpaqueTypeKind::Array => todo!()
						}
					} else {
						unreachable!()
					}
				}
				Self::Transparent { name, fields, sum_type } => {
					match sum_type {
						false => format!("struct {name} {{ {} }}", fields.iter().map(|(fname, ftype)| format!("{fname}: {ftype}")).collect::<Vec<String>>().join(", ")),
						true => format!("enum {name} {{ {} }}", fields.iter().map(|(fname, ftype)| format!("{fname} {ftype}")).collect::<Vec<String>>().join(", ")),
					}
				}
				Self::Reference { to } => format!("&{to}"),
				Self::Generic { name } => name.to_string(),
				Self::Function { name, effect } => format!("{name} {effect}")
			}
		})
	}
}