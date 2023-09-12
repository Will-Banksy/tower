use std::{collections::HashMap, fmt::{Display, Write}};

use crate::{parser::ASTNode, lexer::Literal};

#[derive(PartialEq, Clone, Debug)]
pub enum PrimitiveType {
	U64,
	I64,
	F64,
	Bool,
	StrPtr,
	FnPtr
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PrimitiveType::U64 => "u64",
            PrimitiveType::I64 => "i64",
            PrimitiveType::F64 => "f64",
            PrimitiveType::Bool => "bool",
            PrimitiveType::StrPtr => "strptr",
            PrimitiveType::FnPtr => "fnptr",
        })
    }
}

#[derive(Clone, Debug)]
pub struct StackEffect {
	popped: im::Vector<PrimitiveType>,
	pushed: im::Vector<PrimitiveType>
}

impl StackEffect {
	fn new(popped: im::Vector<PrimitiveType>, pushed: im::Vector<PrimitiveType>) -> Self {
		StackEffect { popped, pushed }
	}

	fn new_popped(popped: im::Vector<PrimitiveType>) -> Self {
		Self::new(popped, im::Vector::new())
	}

	fn new_pushed(pushed: im::Vector<PrimitiveType>) -> Self {
		Self::new(im::Vector::new(), pushed)
	}

	fn none() -> Self {
		StackEffect { popped: im::Vector::new(), pushed: im::Vector::new() }
	}

	fn combine(mut self, mut next: StackEffect) -> Result<StackEffect, String> {
		while self.pushed.len() > 0 && next.popped.len() > 0 {
			let pushed = self.pushed.pop_back().unwrap();
			let popped = next.popped.pop_front().unwrap();
			if pushed != popped {
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

pub fn stack_effect_for(tles: &HashMap<String, ASTNode>, node: &ASTNode) -> Result<StackEffect, String> {
	match node {
		ASTNode::Module(_) => unimplemented!(),
		ASTNode::Keyword(_) => unimplemented!(), // After parsing, there won't be any keywords
		ASTNode::Instruction(_) => todo!(), // TODO: Add some mechanism for declaring the stack effect in the instructions - Although there won't actually be any instructions at this point, they get added in the interpreter
		ASTNode::Function(fn_node) => stack_effect_for(tles, fn_node),
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
				accum = accum.combine(stack_effect_for(tles, node)?)?;
			}
			Ok(accum)
		},
		ASTNode::Word(word) => tles.get(word).map(|func| stack_effect_for(tles, func)).unwrap_or(Err(format!("[ERROR]: No function {}", word)))
	}
}

fn analyse(ast: &ASTNode) {
}