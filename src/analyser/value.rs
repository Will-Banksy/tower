use std::{fmt::Display, rc::Rc};

use crate::{analyser::ttype::OpaqueTypeKind, parser::tree::Literal};

use super::{tree::TypedTreeNode, ttype::Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
	ty: Type,
	inner: ValueInner
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueInner {
	Bytes(im::Vector<u8>),
	Reference {
		to: Rc<Value>
	},
	Function {
		node: Rc<TypedTreeNode>
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

	pub fn from_lit(lit: &Literal) -> Value {
		match lit {
			Literal::U128(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::U64(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::U32(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::U16(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::U8(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::I128(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::I64(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::I32(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::I16(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::I8(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::F64(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::F32(val) => Value::from_typed_bytes(Type::from_lit(lit), val.to_ne_bytes()),
			Literal::Bool(val) => Value::from_typed_bytes(Type::from_lit(lit), [*val as u8]),
			Literal::String(val) => Value::new_reference(Value::from_typed_bytes(Type::new_str(val.len()), val.bytes())),
			Literal::FnPtr(val) => todo!(),
		}
	}

	pub fn to_lit(&self) -> Value { // TODO: ?
		todo!()
		// match self.ty {
		// 	Type::Opaque { size, kind } => {
		// 		match kind {
		// 			OpaqueTypeKind::UnsignedInt => todo!(),
		// 			OpaqueTypeKind::SignedInt => todo!(),
		// 			OpaqueTypeKind::Float => todo!(),
		// 			OpaqueTypeKind::Bool => todo!(),
		// 			OpaqueTypeKind::Str => todo!(),
		// 			OpaqueTypeKind::Array => todo!(),
		// 		}
		// 		todo!()
		// 	},
		// 	Type::Transparent { name, fields, sum_type } => todo!(),
		// 	Type::Reference { to } => todo!(),
		// 	Type::Generic { name } => todo!(),
		// 	Type::Function { name, effect } => todo!(),
		// }
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { // TODO: ?
		todo!()
	}
}