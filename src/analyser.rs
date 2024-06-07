use std::{collections::HashMap, fmt::{Display, Write}};

use crate::{instructions::instructions, lexer::Literal, parser::{ASTNode, AnnotatedASTNode, Instruction, NodeId}};

#[derive(PartialEq, Clone, Debug)]
pub enum PrimitiveType {
	U64,
	I64,
	F64,
	Bool,
	StrPtr,
	FnPtr,
	Generic(String)
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let gen_fmt_str;
        let string = match self {
            PrimitiveType::U64 => "u64",
            PrimitiveType::I64 => "i64",
            PrimitiveType::F64 => "f64",
            PrimitiveType::Bool => "bool",
            PrimitiveType::StrPtr => "strptr",
            PrimitiveType::FnPtr => "fnptr",
			PrimitiveType::Generic(tident) => {
				gen_fmt_str = format!("<{}>", tident);
				&gen_fmt_str
			},
        };
		write!(f, "{}", string)
    }
}

#[derive(Clone, Debug)]
pub struct StackEffect {
	popped: im::Vector<PrimitiveType>,
	pushed: im::Vector<PrimitiveType>
}

impl StackEffect {
	pub fn new(popped: im::Vector<PrimitiveType>, pushed: im::Vector<PrimitiveType>) -> Self {
		StackEffect { popped, pushed }
	}

	pub fn new_popped(popped: im::Vector<PrimitiveType>) -> Self {
		Self::new(popped, im::Vector::new())
	}

	pub fn new_pushed(pushed: im::Vector<PrimitiveType>) -> Self {
		Self::new(im::Vector::new(), pushed)
	}

	pub fn none() -> Self {
		StackEffect { popped: im::Vector::new(), pushed: im::Vector::new() }
	}

	pub fn combine(mut self, mut next: StackEffect) -> Result<StackEffect, String> { // TODO: Check/test this function to see whether it is correct
		while self.pushed.len() > 0 && next.popped.len() > 0 {
			let pushed = self.pushed.pop_back().unwrap();
			let popped = next.popped.pop_front().unwrap();
			if pushed == popped {
				() // good, true
			} else if let PrimitiveType::Generic(_) = pushed {
				// TODO
			} else if let PrimitiveType::Generic(_) = popped {
				// TODO
			} else {
				return Err(format!("[ERROR]: Incompatible types {:?} and {:?}", pushed, popped))
			}
		}

		self.popped.append(next.popped);
		self.pushed.append(next.pushed);

		Ok(self)
	}
}

impl Display for StackEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut popped_sb = String::new();
		let mut drain = false;
		for pop in &self.popped {
			if !drain {
				popped_sb.push(' ');
			}
			write!(popped_sb, "{}, ", pop)?;
			drain = true;
		}
		if drain {
			popped_sb.drain((popped_sb.len() - 2)..popped_sb.len());
		}

		let mut pushed_sb = String::new();
		drain = false;
		for push in &self.pushed {
			write!(pushed_sb, "{}, ", push)?;
			drain = true;
		}
		if drain {
			pushed_sb.drain((pushed_sb.len() - 2)..pushed_sb.len());
			pushed_sb.push(' ')
		}

        write!(f, "({} -> {})", popped_sb, pushed_sb)
    }
}

/// Walk recursively through the AST, starting at node, calculating stack effects.
// BUG: Cannot handle recursive patterns, i.e. a function calling itself
pub fn calc_stack_effects(tles: &HashMap<String, AnnotatedASTNode>, node: &AnnotatedASTNode, effects: &mut HashMap<NodeId, StackEffect>) -> Result<StackEffect, String> {
	if let Some(effect) = effects.get(&node.id) {
		return Ok(effect.clone());
	}

	let effect = match &node.node {
		ASTNode::Module(tles, entry_point) => {
			if let Some(entry_fn) = tles.get(entry_point) {
				calc_stack_effects(&tles, entry_fn, effects)
			} else {
				Err(format!("[ERROR]: Entry point \"{}\" not found", entry_point))
			}
		},
		ASTNode::Keyword(_) => unimplemented!(), // After parsing, there won't be any keywords
		ASTNode::Instruction(_) => unimplemented!(), // No instructions at this point, and if there were execution wouldn't reach here anyway cause all instructions have effects from the start
		ASTNode::Function(fn_node) => calc_stack_effects(tles, &fn_node, effects),
		ASTNode::Literal(lit) => {
			match lit {
				Literal::U64(_) => Ok(StackEffect::new_pushed(im::vector![PrimitiveType::U64])),
				Literal::I64(_) => Ok(StackEffect::new_pushed(im::vector![PrimitiveType::I64])),
				Literal::F64(_) => Ok(StackEffect::new_pushed(im::vector![PrimitiveType::F64])),
				Literal::Bool(_) => Ok(StackEffect::new_pushed(im::vector![PrimitiveType::Bool])),
				Literal::String(_) => Ok(StackEffect::new_pushed(im::vector![PrimitiveType::StrPtr])),
				Literal::FnPtr(_) => Ok(StackEffect::new_pushed(im::vector![PrimitiveType::FnPtr])),
			}
		},
		ASTNode::Block(nodes) => {
			let mut accum = StackEffect::none();
			for node in nodes {
				accum = accum.combine(calc_stack_effects(tles, &node, effects)?)?;
			}
			Ok(accum)
		},
		ASTNode::Word(word) => tles.get(word).map(|func| calc_stack_effects(tles, func, effects)).unwrap_or(Err(format!("[ERROR]: No function {}", word))) // NOTE: Will fail at instruction names if they are not added with stack effects at this point
	}?;
	effects.insert(node.id, effect.clone());
	Ok(effect)
}

/// Performs semantic analysis - Defines instructions and checks stack effects
pub fn analyse(ast: &mut AnnotatedASTNode, node_id: &mut NodeId) -> Result<HashMap<NodeId, StackEffect>, String> {
	// TODO: ALSO need to do monomorphisation and figure out generics

	let mut effects: HashMap<NodeId, StackEffect> = HashMap::new();

	if let ASTNode::Module(tles, _) = &mut ast.node {
		add_instructions(tles, &mut effects, node_id);

		for (_, tle) in &*tles {
			calc_stack_effects(tles, tle, &mut effects)?;
		}
	}

	Ok(effects)

	// todo!()

	// if let ASTNode::Module(mut tles, entry_point) = ast {
	// 	add_instructions(&mut tles);
	// 	if let Some(entry_point) = entry_point {
	// 		if let Some(entry_fn) = tles.get(&entry_point) {
	// 			let effect = stack_effect_for(&tles, entry_fn)?;
	// 			println!("\n\nENTRY POINT STACK EFFECT: {}", effect);
	// 			Ok(ast.clone().annotated(effect))
	// 		} else {
	// 			Err(format!("[ERROR]: Entry point \"{}\" not found", entry_point))
	// 		}
	// 	} else {
	// 		unimplemented!()
	// 	}
	// } else {
	// 	Err("[ERROR]: Not a module".into())
	// }

	// if let ASTNode::Module(tles, entry_point) = ast {

	// }
}

fn add_instructions(program: &mut HashMap<String, AnnotatedASTNode>, effects: &mut HashMap<NodeId, StackEffect>, node_id: &mut NodeId) {
	let instructions: im::HashMap<String, (Instruction, StackEffect)> = instructions();

	for (instruct_name, (instruct_body, instruct_effect)) in instructions {
		program.insert(instruct_name.to_string(), AnnotatedASTNode::new(ASTNode::Instruction(instruct_body), node_id.inc()));
		effects.insert(*node_id, instruct_effect);
	}
}