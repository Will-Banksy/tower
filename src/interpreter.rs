use std::{collections::HashMap, rc::Rc};

use crate::{parser::{ASTNode, Instruction}, lexer::Literal};

#[derive(Debug, Clone, PartialEq)]
pub enum StackItem { // TODO: Each StackItem is currently 24 bytes. This is not ideal.
	U64(u64),
	I64(i64),
	F64(f64),
	Bool(bool),
	StrPtr(Rc<str>), // TODO: Optimise handling of strings - Store each equal string once and pass around Rcs or better yet does im have structurally shared strings...
	FnPtr(Rc<str>),
}

pub fn interp(program: ASTNode) -> Result<(), String> { // TODO: Make the tower language compatible with and interpreter able to run as a REPL
	if let ASTNode::Module(mut mod_content) = program {
		add_instructions(&mut mod_content);

		let mut stack: Vec<StackItem> = Vec::new();

		exec_fn(&mut stack, &mod_content, "main")?;

		Ok(())
	} else {
		Err("[ERROR]: Not a module".to_string())
	}
}

fn exec_fn(stack: &mut Vec<StackItem>, fns: &HashMap<String, ASTNode>, fn_name: &str) -> Result<(), String> {
	let func = fns.get(fn_name).ok_or(format!("[ERROR]: No function with name: \"{}\"", fn_name))?;

	exec_node(stack, fns, func)

	// match func {
	// 	ASTNode::Function(fn_body) => {
	// 		for node in fn_body {
	// 			let res = exec_node(stack, fns, node);
	// 			if res.is_err() {
	// 				return res;
	// 			}
	// 		}
	// 		Ok(())
	// 	},
	// 	ASTNode::Instruction(func) => func(stack, fns),
	// 	_ => unimplemented!("[PARSER ERROR]: Top level item should be either a function or instruction")
	// }
}

fn exec_node(stack: &mut Vec<StackItem>, fns: &HashMap<String, ASTNode>, node: &ASTNode) -> Result<(), String> {
	match node {
		ASTNode::Module(_) => unimplemented!(),
		ASTNode::Function(node) => exec_node(stack, fns, node),
		ASTNode::Keyword(_) => unimplemented!(),
		ASTNode::Literal(lit) => {
			let item = lit_to_stackitem(lit);
			stack.push(item);
			Ok(())
		},
		ASTNode::Word(word) => exec_fn(stack, fns, word),
		ASTNode::Instruction(func) => func(stack, fns),
		ASTNode::Block(blck_body) => {
			for node in blck_body {
				let res = exec_node(stack, fns, node);
				if res.is_err() {
					return res;
				}
			}
			Ok(())
		}
	}
}

fn lit_to_stackitem(lit: &Literal) -> StackItem {
	match lit {
		Literal::U64(n) => StackItem::U64(*n),
		Literal::I64(n) => StackItem::I64(*n),
		Literal::F64(n) => StackItem::F64(*n),
		Literal::Bool(b) => StackItem::Bool(*b),
		Literal::String(s) => StackItem::StrPtr(s.clone().into()),
		Literal::FnPtr(f) => StackItem::FnPtr(f.clone().into()),
	}
}

fn add_instructions(program: &mut HashMap<String, ASTNode>) {
	const ERR_EMPTY: &'static str = "[ERROR]: Stack empty";

	let instructions: HashMap<String, Instruction> = HashMap::from([
		(
			"print".into(),
			Box::new(|stack: &mut Vec<StackItem>, _: &HashMap<String, ASTNode>| -> Result<(), String> {
				match stack.pop().ok_or(ERR_EMPTY)? {
					StackItem::U64(n) => { print!("{}", n); Ok(()) },
					StackItem::I64(n) => { print!("{}", n); Ok(()) },
					StackItem::F64(n) => { print!("{}", n); Ok(()) },
					StackItem::Bool(b) => { print!("{}", b); Ok(()) }
					StackItem::StrPtr(s) => { print!("{}", s); Ok(()) },
					StackItem::FnPtr(f) => { print!("&{}", f); Ok(()) }
				}
			}) as Instruction
		),
		(
			"println".into(),
			Box::new(|stack: &mut Vec<StackItem>, _: &HashMap<String, ASTNode>| -> Result<(), String> {
				match stack.pop().ok_or(ERR_EMPTY)? {
					StackItem::U64(n) => { println!("{}", n); Ok(()) },
					StackItem::I64(n) => { println!("{}", n); Ok(()) },
					StackItem::F64(n) => { println!("{}", n); Ok(()) },
					StackItem::Bool(b) => { println!("{}", b); Ok(()) }
					StackItem::StrPtr(s) => { println!("{}", s); Ok(()) },
					StackItem::FnPtr(f) => { println!("&{}", f); Ok(()) }
				}
			}) as Instruction
		),
		(
			"call".into(),
			Box::new(|stack: &mut Vec<StackItem>, context: &HashMap<String, ASTNode>| -> Result<(), String> {
				match stack.pop().ok_or(ERR_EMPTY)? {
					StackItem::FnPtr(f) => {
						exec_fn(stack, &context, &f)
					},
					item => Err(format!("[ERROR]: Expected type fnptr, got {:?}", item))
				}
			}) as Instruction
		),
		(
			"dup".into(),
			Box::new(|stack: &mut Vec<StackItem>, _: &HashMap<String, ASTNode>| -> Result<(), String> {
				stack.push(stack.last().ok_or(ERR_EMPTY)?.clone());
				Ok(())
			}) as Instruction
		),
		(
			"eq".into(),
			Box::new(|stack: &mut Vec<StackItem>, _: &HashMap<String, ASTNode>| -> Result<(), String> {
				let lhs = stack.pop().ok_or(ERR_EMPTY)?;
				let rhs = stack.pop().ok_or(ERR_EMPTY)?;
				let res = lhs == rhs;
				stack.push(StackItem::Bool(res));
				Ok(())
			}) as Instruction
		),
		(
			"ne".into(),
			Box::new(|stack: &mut Vec<StackItem>, _: &HashMap<String, ASTNode>| -> Result<(), String> {
				let lhs = stack.pop().ok_or(ERR_EMPTY)?;
				let rhs = stack.pop().ok_or(ERR_EMPTY)?;
				let res = lhs != rhs;
				stack.push(StackItem::Bool(res));
				Ok(())
			}) as Instruction
		),
		(
			"if".into(),
			Box::new(|stack: &mut Vec<StackItem>, context: &HashMap<String, ASTNode>| -> Result<(), String> {
				let fnptr = stack.pop().ok_or(ERR_EMPTY)?;
				let cond = stack.pop().ok_or(ERR_EMPTY)?;
				if let StackItem::Bool(val) = cond {
					if val {
						if let StackItem::FnPtr(fn_name) = fnptr {
							return exec_fn(stack, context, &fn_name)
						}
						return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr))
					}
					return Ok(());
				}
				Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
			}) as Instruction
		),
		(
			"ifelse".into(),
			Box::new(|stack: &mut Vec<StackItem>, context: &HashMap<String, ASTNode>| -> Result<(), String> {
				let fnptr_else = stack.pop().ok_or(ERR_EMPTY)?;
				let fnptr_if = stack.pop().ok_or(ERR_EMPTY)?;
				let cond = stack.pop().ok_or(ERR_EMPTY)?;

				if let StackItem::Bool(val) = cond {
					if val {
						if let StackItem::FnPtr(fn_name) = fnptr_if {
							return exec_fn(stack, context, &fn_name);
						}
						return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr_if))
					} else {
						if let StackItem::FnPtr(fn_name) = fnptr_else {
							return exec_fn(stack, context, &fn_name);
						}
						return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr_else));
					}
				}
				Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
			}) as Instruction
		),
		(
			"while".into(),
			Box::new(|stack: &mut Vec<StackItem>, context: &HashMap<String, ASTNode>| -> Result<(), String> {
				let fnptr = stack.pop().ok_or(ERR_EMPTY)?;
				let cond = stack.pop().ok_or(ERR_EMPTY)?;

				if let StackItem::Bool(val) = cond {
					if val {
						if let StackItem::FnPtr(fn_name) = fnptr {
							loop {
								let res = exec_fn(stack, context, &fn_name);
								if res.is_err() {
									return res;
								}
								let cond = stack.pop().ok_or(ERR_EMPTY)?;
								if let StackItem::Bool(val) = cond {
									if !val {
										return Ok(());
									}
								} else {
									return Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
								}
							}
						}
						return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr))
					}
					return Ok(());
				}
				Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
			}) as Instruction
		),
		(
			"add".into(),
			Box::new(|stack: &mut Vec<StackItem>, _: &HashMap<String, ASTNode>| -> Result<(), String> {
				let lhs = stack.pop().ok_or(ERR_EMPTY)?;
				let rhs = stack.pop().ok_or(ERR_EMPTY)?;
				match lhs {
					StackItem::U64(lhs) => {
						if let StackItem::U64(rhs) = rhs {
							stack.push(StackItem::U64(lhs + rhs));
							Ok(())
						} else {
							Err(format!("[ERROR]: Expected type u64, got {:?}", rhs))
						}
					},
					StackItem::I64(lhs) => {
						if let StackItem::I64(rhs) = rhs {
							stack.push(StackItem::I64(lhs + rhs));
							Ok(())
						} else {
							Err(format!("[ERROR]: Expected type i64, got {:?}", rhs))
						}
					},
					StackItem::F64(lhs) => {
						if let StackItem::F64(rhs) = rhs {
							stack.push(StackItem::F64(lhs + rhs));
							Ok(())
						} else {
							Err(format!("[ERROR]: Expected type f64, got {:?}", rhs))
						}
					},
					lhs => Err(format!("[ERROR]: Expected addable type (u64, i64, f64), got {:?}", lhs))
				}
			}) as Instruction
		)
	]);

	for (instruct_name, instruct_body) in instructions {
		program.insert(instruct_name.to_string(), ASTNode::Instruction(instruct_body));
	}
}