use super::{stack_effect::StackEffect, ttype::Type, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct TypedTreeNode {
	pub file_path: String,
	pub cursor: usize,
	pub tree: TypedTree
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedTree {
	Module {
		name: String,
		elems: im::OrdMap<String, TypedTreeNode>
	},
	Function {
		name: String,
		effect: StackEffect,
		body: im::Vector<TypedTreeNode>,
	},
	Type(Type),
	Word(String),
	Literal {
		ty: Type,
		value: Value
	},
	Constructor {
		ty: Type,
		effect: StackEffect
	}
}

impl TypedTree {
	pub fn wrap(self, file_path: impl Into<String>, cursor: usize) -> TypedTreeNode {
		TypedTreeNode {
			file_path: file_path.into(),
			cursor,
			tree: self
		}
	}
}