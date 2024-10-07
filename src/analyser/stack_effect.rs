use std::fmt::{Display, Write};

use crate::{error::{SyntaxError, SyntaxErrorKind}, parser::tree::ParseTreeType};

use super::ttype::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct StackEffect {
	pushed: im::Vector<Type>,
	popped: im::Vector<Type>
}

impl StackEffect {
	pub fn new(popped: im::Vector<Type>, pushed: im::Vector<Type>) -> Self {
		StackEffect { popped, pushed }
	}

	pub fn new_popped(popped: im::Vector<Type>) -> Self {
		Self::new(popped, im::Vector::new())
	}

	pub fn new_pushed(pushed: im::Vector<Type>) -> Self {
		Self::new(im::Vector::new(), pushed)
	}

	pub fn none() -> Self {
		StackEffect { popped: im::Vector::new(), pushed: im::Vector::new() }
	}

	pub fn new_constructor(of: Type, fields: &im::OrdMap<String, Type>) -> StackEffect {
		let mut popped = im::Vector::new();

		for (_, ftype) in fields {
			popped.push_back(ftype.clone());
		}

		StackEffect::new(popped, im::vector![of])
	}

	// TODO: Turn this shit into something usable with the new way of doing things
	//       Essentially, this will help craft the type system
	//       I think we need context however - Whether a type can "become" a generic type is dependent on the required functions to be defined
	pub fn combine(mut self, next: &StackEffect) -> Result<StackEffect, SyntaxError> {
		let mut next = next.clone();
		while self.pushed.len() > 0 && next.popped.len() > 0 {
			let pushed = self.pushed.pop_back().unwrap();
			let popped = next.popped.pop_front().unwrap();
			if pushed == popped {
				() // good, true
			} else if let Type::Generic { name } = pushed {
				if let Type::Generic { name } = popped {
					// TODO: Check whether generic types are compatible
				}
				// TODO: Check whether pushed generic type is compatible with popped concrete type - Or instead delegate this decision to the popper

				todo!() // NOTE: Generics are temporarily disabled
			} else if let Type::Generic { name } = popped {
				// TODO: Check whether the pushed concrete type is compatible with the popped generic type

				todo!()
			} else {
				return Err(SyntaxError::new(SyntaxErrorKind::IncompatibleTypes { source: pushed, dest: popped }, ParseTreeType::None, 0))
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