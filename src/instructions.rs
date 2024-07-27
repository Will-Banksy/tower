use std::{collections::HashMap, rc::Rc};

use crate::{analyser::{StackEffect, TowerType}, error::RuntimeError, interpreter::exec_fn, parser::{AnnotatedASTNode, Instruction}, stack::TowerStack};

pub fn instructions() -> im::HashMap<String, (Instruction, StackEffect)> {
	im::hashmap! {
		"println_str".into() => (
			Rc::new(|stack: &mut Box<dyn TowerStack>, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), RuntimeError> {
				let string = stack.pop_strptr()?;
				println!("{string}");

				Ok(())
			}) as Instruction,
			StackEffect::new_popped(im::vector![TowerType::StrPtr])
		),
		"call".into() => (
			Rc::new(|stack: &mut Box<dyn TowerStack>, symbols: &HashMap<String, AnnotatedASTNode>| -> Result<(), RuntimeError> {
				let string = stack.pop_fnptr()?;
				exec_fn(stack, symbols, &string)?;

				Ok(())
			}) as Instruction,
			StackEffect::new_popped(im::vector![TowerType::FnPtr])
		),
		"dup_str".into() => (
			Rc::new(|stack: &mut Box<dyn TowerStack>, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), RuntimeError> {
				let string = stack.pop_strptr()?;
				stack.push_strptr(string.clone())?;
				stack.push_strptr(string)?;

				Ok(())
			}) as Instruction,
			StackEffect::new(im::vector![TowerType::StrPtr], im::vector![TowerType::StrPtr, TowerType::StrPtr])
		),
		"if".into() => (
			Rc::new(|stack: &mut Box<dyn TowerStack>, symbols: &HashMap<String, AnnotatedASTNode>| -> Result<(), RuntimeError> {
				let fnptr = stack.pop_fnptr()?;
				let cond = stack.pop_bool()?;
				if cond {
					exec_fn(stack, symbols, &fnptr)?;
				}

				Ok(())
			}) as Instruction,
			StackEffect::new_popped(im::vector![TowerType::FnPtr])
		)
	}

	// im::hashmap! {
	// 	"print".into() => (
	// 		Rc::new(|stack: &mut S, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			match stack.pop().ok_or(ERR_EMPTY)? {
	// 				StackItem::U64(n) => { print!("{}", n); Ok(()) },
	// 				StackItem::I64(n) => { print!("{}", n); Ok(()) },
	// 				StackItem::F64(n) => { print!("{}", n); Ok(()) },
	// 				StackItem::Bool(b) => { print!("{}", b); Ok(()) }
	// 				StackItem::StrPtr(s) => { print!("{}", s); Ok(()) },
	// 				StackItem::FnPtr(f) => { print!("&{}", f); Ok(()) }
	// 			}
	// 		}) as Instruction<S>,
	// 		StackEffect::new_popped(im::vector![TowerType::Generic("T".into())])
	// 	),
	// 	"println".into() => (
	// 		Rc::new(|stack: &mut S, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			match stack.pop().ok_or(ERR_EMPTY)? {
	// 				StackItem::U64(n) => { println!("{}", n); Ok(()) },
	// 				StackItem::I64(n) => { println!("{}", n); Ok(()) },
	// 				StackItem::F64(n) => { println!("{}", n); Ok(()) },
	// 				StackItem::Bool(b) => { println!("{}", b); Ok(()) }
	// 				StackItem::StrPtr(s) => { println!("{}", s); Ok(()) },
	// 				StackItem::FnPtr(f) => { println!("&{}", f); Ok(()) }
	// 			}
	// 		}) as Instruction,
	// 		StackEffect::new_popped(im::vector![TowerType::Generic("T".into())])
	// 	),
	// 	"call".into() => (
	// 		Rc::new(|stack: &mut S, context: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			match stack.pop().ok_or(ERR_EMPTY)? {
	// 				StackItem::FnPtr(f) => {
	// 					exec_fn(stack, &context, &f)
	// 				},
	// 				item => Err(format!("[ERROR]: Expected type fnptr, got {:?}", item))
	// 			}
	// 		}) as Instruction,
	// 		StackEffect::new_popped(im::vector![TowerType::FnPtr])
	// 	),
	// 	"dup".into() => (
	// 		Rc::new(|stack: &mut S, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			stack.push(stack.last().ok_or(ERR_EMPTY)?.clone());
	// 			Ok(())
	// 		}) as Instruction,
	// 		StackEffect::new(im::vector![TowerType::Generic("T".into())], im::vector![TowerType::Generic("T".into()), TowerType::Generic("T".into())])
	// 	),
	// 	"eq".into() => (
	// 		Rc::new(|stack: &mut S, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			let lhs = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let rhs = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let res = lhs == rhs;
	// 			stack.push(StackItem::Bool(res));
	// 			Ok(())
	// 		}) as Instruction,
	// 		StackEffect::new_popped(im::vector![TowerType::Generic("L".into()), TowerType::Generic("R".into())])
	// 	),
	// 	"ne".into() => (
	// 		Rc::new(|stack: &mut S, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			let lhs = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let rhs = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let res = lhs != rhs;
	// 			stack.push(StackItem::Bool(res));
	// 			Ok(())
	// 		}) as Instruction,
	// 		StackEffect::new_popped(im::vector![TowerType::Generic("L".into()), TowerType::Generic("R".into())])
	// 	),
	// 	"if".into() => (
	// 		Rc::new(|stack: &mut S, context: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			let fnptr = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let cond = stack.pop().ok_or(ERR_EMPTY)?;
	// 			if let StackItem::Bool(val) = cond {
	// 				if val {
	// 					if let StackItem::FnPtr(fn_name) = fnptr {
	// 						return exec_fn(stack, context, &fn_name)
	// 					}
	// 					return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr))
	// 				}
	// 				return Ok(());
	// 			}
	// 			Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
	// 		}) as Instruction,
	// 		StackEffect::new_popped(im::vector![TowerType::FnPtr, TowerType::Bool])
	// 	),
	// 	"ifelse".into() => (
	// 		Rc::new(|stack: &mut S, context: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			let fnptr_else = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let fnptr_if = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let cond = stack.pop().ok_or(ERR_EMPTY)?;

	// 			if let StackItem::Bool(val) = cond {
	// 				if val {
	// 					if let StackItem::FnPtr(fn_name) = fnptr_if {
	// 						return exec_fn(stack, context, &fn_name);
	// 					}
	// 					return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr_if))
	// 				} else {
	// 					if let StackItem::FnPtr(fn_name) = fnptr_else {
	// 						return exec_fn(stack, context, &fn_name);
	// 					}
	// 					return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr_else));
	// 				}
	// 			}
	// 			Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
	// 		}) as Instruction,
	// 		StackEffect::new_popped(im::vector![TowerType::FnPtr, TowerType::FnPtr, TowerType::Bool])
	// 	),
	// 	"while".into() => (
	// 		Rc::new(|stack: &mut S, context: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			let fnptr = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let cond = stack.pop().ok_or(ERR_EMPTY)?;

	// 			if let StackItem::Bool(val) = cond {
	// 				if val {
	// 					if let StackItem::FnPtr(fn_name) = fnptr {
	// 						loop {
	// 							let res = exec_fn(stack, context, &fn_name);
	// 							if res.is_err() {
	// 								return res;
	// 							}
	// 							let cond = stack.pop().ok_or(ERR_EMPTY)?;
	// 							if let StackItem::Bool(val) = cond {
	// 								if !val {
	// 									return Ok(());
	// 								}
	// 							} else {
	// 								return Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
	// 							}
	// 						}
	// 					}
	// 					return Err(format!("[ERROR]: Expected type fnptr, got {:?}", fnptr))
	// 				}
	// 				return Ok(());
	// 			}
	// 			Err(format!("[ERROR]: Expected type bool, got {:?}", cond))
	// 		}) as Instruction,
	// 		StackEffect::new_popped(im::vector![TowerType::FnPtr, TowerType::Bool])
	// 	),
	// 	"add".into() => (
	// 		Rc::new(|stack: &mut S, _: &HashMap<String, AnnotatedASTNode>| -> Result<(), String> {
	// 			let lhs = stack.pop().ok_or(ERR_EMPTY)?;
	// 			let rhs = stack.pop().ok_or(ERR_EMPTY)?;
	// 			match lhs {
	// 				StackItem::U64(lhs) => {
	// 					if let StackItem::U64(rhs) = rhs {
	// 						stack.push(StackItem::U64(lhs + rhs));
	// 						Ok(())
	// 					} else {
	// 						Err(format!("[ERROR]: Expected type u64, got {:?}", rhs))
	// 					}
	// 				},
	// 				StackItem::I64(lhs) => {
	// 					if let StackItem::I64(rhs) = rhs {
	// 						stack.push(StackItem::I64(lhs + rhs));
	// 						Ok(())
	// 					} else {
	// 						Err(format!("[ERROR]: Expected type i64, got {:?}", rhs))
	// 					}
	// 				},
	// 				StackItem::F64(lhs) => {
	// 					if let StackItem::F64(rhs) = rhs {
	// 						stack.push(StackItem::F64(lhs + rhs));
	// 						Ok(())
	// 					} else {
	// 						Err(format!("[ERROR]: Expected type f64, got {:?}", rhs))
	// 					}
	// 				},
	// 				lhs => Err(format!("[ERROR]: Expected addable type (u64, i64, f64), got {:?}", lhs))
	// 			}
	// 		}) as Instruction,
	// 		StackEffect::new(im::vector![TowerType::Generic("A".into()), TowerType::Generic("A".into())], im::vector![TowerType::Generic("B".into())])
	// 	)
	// }
}