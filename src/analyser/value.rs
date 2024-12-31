use std::{fmt::Display, rc::Rc};

use crate::{analyser::ttype::OpaqueTypeKind, parser::tree::Literal};

use super::{stack_effect::StackEffect, ttype::Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Value { // TODO: A lot of thought needs to go into this - The current Value struct is not useful at compile time
	pub ty: Type,
	pub inner: ValueInner
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueInner {
	Bytes(Vec<u8>),
	Struct(im::Vector<Value>),
	Reference {
		to: Rc<Value>
	},
	Function {
		fn_name: String
	}
}

impl Value {
	pub fn from_typed_bytes(ty: Type, bytes: impl IntoIterator<Item = u8>) -> Value {
		Value {
			ty,
			inner: ValueInner::Bytes(bytes.into_iter().collect())
		}
	}

	pub fn new_reference(to: Value) -> Value {
		Value {
			ty: Type::Reference { to: Box::new(to.ty.clone()) },
			inner: ValueInner::Reference { to: Rc::new(to) }
		}
	}

	pub fn new_fn(fn_name: String, effect: StackEffect) -> Value {
		Value {
			ty: Type::Function { name: fn_name.clone(), effect },
			inner: ValueInner::Function { fn_name }
		}
	}

	pub fn new_struct(ty: Type, values: im::Vector<Value>) -> Value {
		Value {
			ty,
			inner: ValueInner::Struct(values)
		}
	}

	/// Produces a Value from the passed-in literal. Returns None if the literal requires context (e.g. Literal::FnPtr)
	pub fn from_lit(lit: &Literal) -> Option<Value> {
		Some(match lit {
			Literal::U128(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::U64(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::U32(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::U16(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::U8(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::I128(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::I64(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::I32(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::I16(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::I8(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::F64(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::F32(val) => Value::from_typed_bytes(Type::from_lit(lit)?, val.to_ne_bytes()),
			Literal::Bool(val) => Value::from_typed_bytes(Type::from_lit(lit)?, [*val as u8]),
			Literal::String(val) => Value::new_reference(Value::from_typed_bytes(Type::new_str(val.len()), val.bytes())),
			_ => return None,
		})
	}

	pub fn deref<'a>(&'a self) -> &'a Value {
		match &self.inner {
			ValueInner::Reference { to } => &to,
			_ => &self
		}
	}

	pub fn as_strref(&self) -> Option<String> {
		if *self.ty.deref().as_opaque()?.1 != OpaqueTypeKind::Str {
			return None;
		}

		match &self.deref().inner {
			ValueInner::Bytes(b) => {
				String::from_utf8(b.clone().into_iter().collect::<Vec<u8>>()).ok()
			}
			_ => unreachable!()
		}
	}

	pub fn as_bytes(&self) -> Option<&[u8]> {
		match &self.inner {
			ValueInner::Bytes(b) => Some(&b),
			_ => None
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { // TODO: ?
		write!(f, "Value(of_type: {}, value: (unable to be displayed))", self.ty)
	}
}