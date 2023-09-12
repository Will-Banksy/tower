use std::{collections::HashMap, rc::Rc};

use crate::{parser::{ASTNode, Instruction}, lexer::Literal, instructions::instructions, analyser::StackEffect};

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
	if let ASTNode::Module(tles, entry_point) = program {
		let mut stack: Vec<StackItem> = Vec::new();

		if let Some(entry_point) = entry_point {
			exec_fn(&mut stack, &tles, &entry_point)?;
			Ok(())
		} else {
			Err("[ERROR]: No defined entry point".into())
		}
	} else {
		Err("[ERROR]: Not a module".to_string())
	}
}

pub(crate) fn exec_fn(stack: &mut Vec<StackItem>, fns: &HashMap<String, ASTNode>, fn_name: &str) -> Result<(), String> {
	let func = fns.get(fn_name).ok_or(format!("[ERROR]: No function with name: \"{}\"", fn_name))?;

	exec_node(stack, fns, func)
}

pub(crate) fn exec_node(stack: &mut Vec<StackItem>, fns: &HashMap<String, ASTNode>, node: &ASTNode) -> Result<(), String> {
	match node {
		ASTNode::Module(_, _) => unimplemented!(),
		ASTNode::Function(node) => exec_node(stack, fns, node),
		ASTNode::Keyword(_) => unimplemented!(),
		ASTNode::Literal(lit) => {
			let item = lit_to_stackitem(lit);
			stack.push(item);
			Ok(())
		},
		ASTNode::Word(word) => exec_fn(stack, fns, word),
		ASTNode::Instruction(func, _) => func(stack, fns),
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