pub mod tree;
pub mod stack_effect;
pub mod ttype;
pub mod value;
pub mod error;

use std::fmt::Display;

use error::{AnalysisError, AnalysisErrorKind};
use stack_effect::StackEffect;
use tree::{TypedTree, TypedTreeNode};
use ttype::Type;
use value::Value;

use crate::{brk, parser::{result::ScanResult::{self, Unrecognised, Valid, WithErr}, tree::{Literal, ParseTree, ParseTreeNode, ParseTreeType}}};

// NOTE: I don't like this
#[derive(PartialEq, Clone, Debug)]
pub enum TowerType {
	U128,
	U64,
	U32,
	U16,
	U8,
	I128,
	I64,
	I32,
	I16,
	I8,
	F64,
	F32,
	Bool,
	StrPtr,
	FnPtr,
	Generic(String)
}

impl Display for TowerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let gen_fmt_str;
        let string = match self {
            TowerType::U128 => "u128",
            TowerType::U64 => "u64",
            TowerType::U32 => "u32",
            TowerType::U16 => "u16",
            TowerType::U8 => "u8",
            TowerType::I128 => "i128",
            TowerType::I64 => "i64",
            TowerType::I32 => "i32",
            TowerType::I16 => "i16",
            TowerType::I8 => "i8",
            TowerType::F64 => "f64",
            TowerType::F32 => "f32",
            TowerType::Bool => "bool",
            TowerType::StrPtr => "strptr",
            TowerType::FnPtr => "fnptr",
			TowerType::Generic(tident) => {
				gen_fmt_str = format!("<{}>", tident);
				&gen_fmt_str
			},
        };
		write!(f, "{}", string)
    }
}

// #[derive(Clone, Debug)]
// pub struct StackEffect {
// 	popped: im::Vector<TowerType>,
// 	pushed: im::Vector<TowerType>
// }

// impl StackEffect {
// 	pub fn new(popped: im::Vector<TowerType>, pushed: im::Vector<TowerType>) -> Self {
// 		StackEffect { popped, pushed }
// 	}

// 	pub fn new_popped(popped: im::Vector<TowerType>) -> Self {
// 		Self::new(popped, im::Vector::new())
// 	}

// 	pub fn new_pushed(pushed: im::Vector<TowerType>) -> Self {
// 		Self::new(im::Vector::new(), pushed)
// 	}

// 	pub fn none() -> Self {
// 		StackEffect { popped: im::Vector::new(), pushed: im::Vector::new() }
// 	}

// 	pub fn combine(mut self, mut next: StackEffect) -> Result<StackEffect, String> {
// 		while self.pushed.len() > 0 && next.popped.len() > 0 {
// 			let pushed = self.pushed.pop_back().unwrap();
// 			let popped = next.popped.pop_front().unwrap();
// 			if pushed == popped {
// 				() // good, true
// 			} else if let TowerType::Generic(_) = pushed {
// 				if let TowerType::Generic(_) = popped {
// 					// TODO: Check whether generic types are compatible
// 				}
// 				// TODO: Check whether pushed generic type is compatible with popped concrete type - Or instead delegate this decision to the popper

// 				todo!() // NOTE: Generics are temporarily disabled
// 			} else if let TowerType::Generic(_) = popped {
// 				// TODO: Check whether the pushed concrete type is compatible with the popped generic type

// 				todo!()
// 			} else {
// 				return Err(format!("[ERROR]: Incompatible types {:?} and {:?}", pushed, popped))
// 			}
// 		}

// 		self.popped.append(next.popped);
// 		self.pushed.append(next.pushed);

// 		Ok(self)
// 	}
// }

// impl Display for StackEffect {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		let mut popped_sb = String::new();
// 		let mut drain = false;
// 		for pop in &self.popped {
// 			if !drain {
// 				popped_sb.push(' ');
// 			}
// 			write!(popped_sb, "{}, ", pop)?;
// 			drain = true;
// 		}
// 		if drain {
// 			popped_sb.drain((popped_sb.len() - 2)..popped_sb.len());
// 		}

// 		let mut pushed_sb = String::new();
// 		drain = false;
// 		for push in &self.pushed {
// 			write!(pushed_sb, "{}, ", push)?;
// 			drain = true;
// 		}
// 		if drain {
// 			pushed_sb.drain((pushed_sb.len() - 2)..pushed_sb.len());
// 			pushed_sb.push(' ')
// 		}

//         write!(f, "({} -> {})", popped_sb, pushed_sb)
//     }
// }

/// Walk recursively through the AST, starting at node, calculating stack effects.
// BUG: Cannot handle recursive patterns, i.e. a function calling itself
// pub fn calc_stack_effects(tles: &OrdMap<String, AnnotatedASTNode>, node: &AnnotatedASTNode, effects: &mut OrdMap<NodeId, StackEffect>) -> Result<StackEffect, String> {
// 	if let Some(effect) = effects.get(&node.id) {
// 		return Ok(effect.clone());
// 	}

// 	let effect = match &node.node {
// 		ASTNode::Module(tles, entry_point) => {
// 			if let Some(entry_fn) = tles.get(entry_point) {
// 				calc_stack_effects(&tles, entry_fn, effects)
// 			} else {
// 				Err(format!("[ERROR]: Entry point \"{}\" not found", entry_point))
// 			}
// 		},
// 		ASTNode::Keyword(_) => unreachable!(), // After parsing, there won't be any keywords
// 		ASTNode::Instruction(_) => unreachable!(), // No instructions at this point, and if there were execution wouldn't reach here anyway cause all instructions have effects from the start
// 		ASTNode::Function(fn_node) => calc_stack_effects(tles, &fn_node, effects),
// 		ASTNode::Literal(lit) => {
// 			match lit {
// 				Literal::U128(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::U128])),
// 				Literal::U64(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::U64])),
// 				Literal::U32(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::U32])),
// 				Literal::U16(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::U16])),
// 				Literal::U8(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::U8])),
// 				Literal::I128(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::I128])),
// 				Literal::I64(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::I64])),
// 				Literal::I32(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::I32])),
// 				Literal::I16(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::I16])),
// 				Literal::I8(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::I8])),
// 				Literal::F64(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::F64])),
// 				Literal::F32(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::F32])),
// 				Literal::Bool(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::Bool])),
// 				Literal::String(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::StrPtr])),
// 				Literal::FnPtr(_) => Ok(StackEffect::new_pushed(im::vector![TowerType::FnPtr])),
// 			}
// 		},
// 		ASTNode::Block(nodes) => {
// 			let mut accum = StackEffect::none();
// 			for node in nodes {
// 				accum = accum.combine(calc_stack_effects(tles, &node, effects)?)?;
// 			}
// 			Ok(accum)
// 		},
// 		ASTNode::Identifier(word) => tles.get(word).map(|func| calc_stack_effects(tles, func, effects)).unwrap_or(Err(format!("[ERROR]: No function {}", word))) // Will fail at instruction names if they are not added with stack effects at this point
// 	}?;
// 	effects.insert(node.id, effect.clone());
// 	Ok(effect)
// }

type AnalysisResult<T> = ScanResult<T, AnalysisError>; // FIXME: Use new error type

fn literal_effect(lit: &Literal) -> StackEffect {
	match lit {
		Literal::U128(_) => StackEffect::new_pushed(im::vector![Type::new_uint(128)]),
		Literal::U64(_) => StackEffect::new_pushed(im::vector![Type::new_uint(64)]),
		Literal::U32(_) => StackEffect::new_pushed(im::vector![Type::new_uint(32)]),
		Literal::U16(_) => StackEffect::new_pushed(im::vector![Type::new_uint(16)]),
		Literal::U8(_) => StackEffect::new_pushed(im::vector![Type::new_uint(8)]),
		Literal::I128(_) => StackEffect::new_pushed(im::vector![Type::new_int(128)]),
		Literal::I64(_) => StackEffect::new_pushed(im::vector![Type::new_int(64)]),
		Literal::I32(_) => StackEffect::new_pushed(im::vector![Type::new_int(32)]),
		Literal::I16(_) => StackEffect::new_pushed(im::vector![Type::new_int(16)]),
		Literal::I8(_) => StackEffect::new_pushed(im::vector![Type::new_int(8)]),
		Literal::F64(_) => todo!(),
		Literal::F32(_) => todo!(),
		Literal::Bool(_) => StackEffect::new_pushed(im::vector![Type::new_bool()]),
		Literal::String(s) => StackEffect::new_pushed(im::vector![Type::new_strref(s.len())]),
		Literal::FnPtr(_) => todo!(),
	}
}

fn calc_stack_effects(parse_tree: &ParseTreeNode, tles: &im::OrdMap<String, TypedTreeNode>, parse_tree_tles: &im::OrdMap<String, ParseTreeNode>) -> AnalysisResult<TypedTreeNode> {
	let tree = match &parse_tree.tree {
		ParseTree::Module { name, elems } => {
			let mut typed_elems = im::OrdMap::new();
			let mut to_analyse: Vec<(&String, &ParseTreeNode)> = elems.into_iter().collect();

			// FIXME: Will currently infinitely loop where there are recursive functions. Check if any functions have been resolved after each iteration and if not then error
			//        Will also infinitely loop where, and this is important, there is a function that refers to another function that has not been analysed yet
			while !to_analyse.is_empty() {
				eprintln!("Checkpoint #0");
				let mut i = 0;
				while i < to_analyse.len() {
					let (name, node) = &to_analyse[i];
					i += 1;
					match calc_stack_effects(&node, &typed_elems, elems) {
						Valid(node) => {
							typed_elems.insert(name.to_string(), node);
							i -= 1;
							to_analyse.remove(i);
						}
						Unrecognised => {
							()
						}
						WithErr(e) => return WithErr(e.clone())
					}
				}
			}

			TypedTree::Module { name: name.to_string(), elems: typed_elems.into_iter().collect() }
		},
		ParseTree::Function { name, body } => {
			let mut effect = StackEffect::none();
			let mut typed_body: im::Vector<TypedTreeNode> = im::Vector::new();

			for elem in body {
				let new_effect = match &elem.tree {
					ParseTree::Identifier(ident) => {
						let func_node = if let Some(func_node) = tles.get(ident) {
							func_node
						} else if parse_tree_tles.contains_key(ident) {
							// If we don't know the effect of a used function (but it exists), return Unrecognised to skip evaluating this function for now
							return Unrecognised;
						} else {
							// If that function doesn't exist, however, we error
							return WithErr(AnalysisError::new(AnalysisErrorKind::NoSuchFunction { fname: ident.to_string() }, 0));
						};

						let ident_effect = match &func_node.tree {
							TypedTree::Function { effect, .. } => effect,
							TypedTree::Type(ty) => return WithErr(AnalysisError::new(AnalysisErrorKind::TypeIsNotFunction { tname: ty.name() }, 0)),
							_ => unreachable!()
						};

						ident_effect.clone()
					},
					ParseTree::Literal(lit) => literal_effect(lit),
					ParseTree::Constructor(ident) => { // FIXME: Code duplication - this and the outer match ParseTree::Constructor case
						let ctype = {
							if let Some(ty) = Type::from_name(ident) {
								ty
							} else {
								if let Some(type_node) = tles.get(ident) {
									match &type_node.tree {
										TypedTree::Type(ty) => ty.clone(),
										_ => unreachable!()
									}
								} else if parse_tree_tles.contains_key(ident) {
									// If we don't know the type of a used type name (but it exists), return Unrecognised to skip evaluating this type for now
									return Unrecognised;
								} else {
									return WithErr(AnalysisError::new(AnalysisErrorKind::NoSuchType { tname: ident.to_string() }, 0))
								}
							}
						};

						let effect = match &ctype {
							Type::Transparent { name: _, fields, sum_type } => { // TODO: Handle sum types (enums)
								StackEffect::new_constructor(ctype.clone(), fields)
							}
							_ => return WithErr(AnalysisError::new(AnalysisErrorKind::UnconstructableType { tname: ctype.name() }, 0))
						};

						effect
					}
					_ => unreachable!() // FIXME: Not unreachable anymore
				};

				typed_body.push_back(brk!(calc_stack_effects(elem, tles, parse_tree_tles)));

				effect = match effect.combine(&new_effect) {
					Ok(effect) => effect,
					Err(e) => return WithErr(e)
				};
			}

			TypedTree::Function { name: name.to_string(), effect, body: typed_body }
		},
		ParseTree::Struct { name, fields } => {
			let mut typed_fields = im::OrdMap::new();

			for (fname, ftype) in fields {
				let typed_ftype = {
					if let Some(ty) = Type::from_name(ftype) {
						ty
					} else {
						if let Some(type_node) = tles.get(ftype) {
							match &type_node.tree {
								TypedTree::Type(ty) => ty.clone(),
								_ => unreachable!()
							}
						} else if parse_tree_tles.contains_key(ftype) {
							// If we don't know the type of a used type name (but it exists), return Unrecognised to skip evaluating this type for now
							return Unrecognised;
						} else {
							return WithErr(AnalysisError::new(AnalysisErrorKind::NoSuchType { tname: ftype.to_string() }, 0))
						}
					}
				};

				typed_fields.insert(fname.to_string(), typed_ftype);
			}

			TypedTree::Type(Type::new_struct(name.to_string(), &typed_fields))
		},
		ParseTree::Enum { name, fields } => todo!(), // TODO
		ParseTree::Identifier(s) => TypedTree::Word(s.to_string()),
		ParseTree::Literal(literal) => TypedTree::Literal { ty: Type::from_lit(literal), value: Value::from_lit(literal) },
		ParseTree::Constructor(ident) => {
			let ctype = {
				if let Some(ty) = Type::from_name(ident) {
					ty
				} else {
					if let Some(type_node) = tles.get(ident) {
						match &type_node.tree {
							TypedTree::Type(ty) => ty.clone(),
							_ => unreachable!()
						}
					} else if parse_tree_tles.contains_key(ident) {
						// If we don't know the type of a used type name (but it exists), return Unrecognised to skip evaluating this type for now
						return Unrecognised;
					} else {
						return WithErr(AnalysisError::new(AnalysisErrorKind::NoSuchType { tname: ident.to_string() }, 0))
					}
				}
			};

			let effect = match &ctype {
				Type::Transparent { name: _, fields, sum_type } => { // TODO: Handle sum types (enums)
					StackEffect::new_constructor(ctype.clone(), fields)
				}
				_ => return WithErr(AnalysisError::new(AnalysisErrorKind::UnconstructableType { tname: ctype.name() }, 0))
			};

			TypedTree::Constructor { ty: ctype, effect }
		}
		ParseTree::FieldAccess(field_name) => todo!(), // TODO
	};

	Valid(
		tree.wrap(parse_tree.file_path.to_string(), parse_tree.cursor)
	)
}

/// Performs semantic analysis - Defines instructions and checks stack effects
pub fn analyse(parse_tree: &ParseTreeNode) -> AnalysisResult<TypedTreeNode> {
	// TODO: ALSO need to do monomorphisation and figure out generics
	// TODO: ALSO need to assign paths to all the relevant things i.e. module::Trait::function, module::function, module::module::module::Struct

	let typed_tree = calc_stack_effects(parse_tree, &im::OrdMap::new(), &im::OrdMap::new());

	typed_tree
}

// fn add_instructions(program: &mut OrdMap<String, AnnotatedASTNode>, effects: &mut OrdMap<NodeId, StackEffect>, node_id: &mut NodeId) {
// 	let instructions: im::OrdMap<String, (Instruction, StackEffect)> = instructions();

// 	for (instruct_name, (instruct_body, instruct_effect)) in instructions {
// 		program.insert(instruct_name.to_string(), AnnotatedASTNode::new(ASTNode::Instruction(instruct_body), node_id.inc()));
// 		effects.insert(*node_id, instruct_effect);
// 	}
// }