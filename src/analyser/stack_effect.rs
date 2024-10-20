use std::fmt::{Display, Write};

use crate::parser::{error::{SyntaxError, SyntaxErrorKind}, tree::{Literal, ParseTreeType}};

use super::{error::{AnalysisError, AnalysisErrorKind}, ttype::Type};

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

	pub fn new_field_access(of: Type, field_type: Type) -> StackEffect {
		StackEffect::new(im::vector![of.clone()], im::vector![of, field_type])
	}

	pub fn last_pushed<'a>(&'a self) -> Option<&'a Type> {
		self.pushed.last()
	}

	/// Returns the stack effect of the passed-in literal, or None if the literal requires context to work out the type (e.g. FnPtr)
	pub fn from_lit(lit: &Literal) -> Option<StackEffect> {
		Some(match lit {
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
			Literal::FnPtr(_) => return None,
		})
	}

	// TODO: Turn this shit into something usable with the new way of doing things
	//       Essentially, this will help craft the type system
	//       I think we need context however - Whether a type can "become" a generic type is dependent on the required functions to be defined
	pub fn combine(mut self, next: &StackEffect) -> Result<StackEffect, AnalysisError> {
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
				return Err(AnalysisError::new(AnalysisErrorKind::IncompatibleTypes { source: pushed, dest: popped }, 0))
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