pub mod error;
pub mod instructions;

use error::{RuntimeError, RuntimeErrorKind};

use crate::analyser::{tree::{TypedTree, TypedTreeNode}, ttype::Type, value::{Value, ValueInner}};

pub fn interp(typed_tree: TypedTreeNode) -> Result<Vec<Value>, RuntimeError> {
	match typed_tree.tree {
		TypedTree::Module { name: _, elems } => {
			let fns: im::OrdMap<String, TypedTreeNode> = elems.iter().filter_map(|(name, e)| if let TypedTree::Function { name: _, effect: _, body: _ } = e.tree { Some((name.clone(), e.clone())) } else { None }).collect();
			let types: im::OrdMap<String, Type> = elems.iter().filter_map(|(name, e)| if let TypedTree::Type(t) = &e.tree { Some((name.clone(), t.clone())) } else { None }).collect();

			if let Some(f) = fns.get("main") {
				let mut stack: Vec<Value> = Vec::new();

				interp_node(f, &fns, &types, &mut stack)?;

				Ok(stack)
			} else {
				return Err(RuntimeError::new(RuntimeErrorKind::FunctionMissingError("main".to_string()), typed_tree.cursor));
			}
		},
		_ => {
			return Err(RuntimeError::new(RuntimeErrorKind::ModuleNotFoundError, typed_tree.cursor))
		}
	}
}

fn interp_node(typed_tree: &TypedTreeNode, fns: &im::OrdMap<String, TypedTreeNode>, types: &im::OrdMap<String, Type>, stack: &mut Vec<Value>) -> Result<(), RuntimeError> {
	match &typed_tree.tree {
		TypedTree::Module { name: _, elems: _ } => unreachable!(),
		TypedTree::Function { name, effect: _, body } => {
			eprintln!("Debug: Executing function {name}");

			for node in body {
				interp_node(node, fns, types, stack)?;
			}

			Ok(())
		},
		TypedTree::Type(_) => unreachable!(),
		TypedTree::Word(wd) => {
			if let Some(node) = fns.get(wd) {
				interp_node(node, fns, types, stack)
			} else {
				return Err(RuntimeError::new(RuntimeErrorKind::FunctionMissingError(wd.clone()), typed_tree.cursor))
			}
		},
		TypedTree::Literal { ty: _, value } => {
			stack.push(value.clone());
			Ok(())
		},
		TypedTree::Constructor { ty, effect: _ } => {
			match ty {
				Type::Transparent { name: _, fields, sum_type } => {
					if *sum_type {
						todo!()
					}

					let mut values = im::Vector::new();
					for (_, ftype) in fields {
						values.push_back(stack.pop().expect("Expected value on stack"));
						assert_eq!(values.last().unwrap().ty, *ftype);
					}

					stack.push(Value::new_struct(ty.clone(), values));

					Ok(())
				},
				_ => unreachable!()
			}
		},
		TypedTree::FieldAccess { name } => {
			if let Some(val) = stack.last() {
				let field_value = if let ValueInner::Struct(vals) = &val.inner {
					if let Type::Transparent { name: _, fields, sum_type: false } = &val.ty {
						vals.iter().zip(fields.iter()).find_map(|(val, (fname, _))| if fname == name { Some(val) } else { None })
					} else {
						unreachable!()
					}
				} else {
					unreachable!()
				};

				if let Some(field_value) = field_value {
					stack.push(field_value.clone());

					Ok(())
				} else {
					unreachable!()
				}
			} else {
				return Err(RuntimeError::new(RuntimeErrorKind::StackUnderflowError, typed_tree.cursor));
			}
		},
	}
}