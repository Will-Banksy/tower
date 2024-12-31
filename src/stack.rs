// use core::slice;
// use std::rc::Rc;

// use crate::{error::RuntimeError, parser::tree::Literal};

// pub trait TowerStack {
// 	fn pop_bool(&mut self) -> Result<bool, RuntimeError>;
// 	fn pop_u64(&mut self) -> Result<u64, RuntimeError>;
// 	fn pop_i64(&mut self) -> Result<i64, RuntimeError>;
// 	fn pop_f64(&mut self) -> Result<f64, RuntimeError>;
// 	fn pop_usize(&mut self) -> Result<usize, RuntimeError>;

// 	fn pop_strptr(&mut self) -> Result<Rc<str>, RuntimeError>;
// 	fn pop_fnptr(&mut self) -> Result<Rc<str>, RuntimeError>;


// 	fn push_bool(&mut self, val: bool) -> Result<(), RuntimeError>;
// 	fn push_u64(&mut self, val: u64) -> Result<(), RuntimeError>;
// 	fn push_i64(&mut self, val: i64) -> Result<(), RuntimeError>;
// 	fn push_f64(&mut self, val: f64) -> Result<(), RuntimeError>;
// 	fn push_usize(&mut self, val: usize) -> Result<(), RuntimeError>;

// 	fn push_strptr(&mut self, val: Rc<str>) -> Result<(), RuntimeError>;
// 	fn push_fnptr(&mut self, val: Rc<str>) -> Result<(), RuntimeError>;

// 	fn push_lit(&mut self, val: &Literal) -> Result<(), RuntimeError>;
// }

// impl TowerStack for Vec<u8> {
// 	fn pop_bool(&mut self) -> Result<bool, RuntimeError> {
// 		let val = self.pop().ok_or(RuntimeError::StackUnderflowError)?;
// 		Ok(val != 0)
// 	}

// 	fn pop_u64(&mut self) -> Result<u64, RuntimeError> {
// 		let slc = self.last_chunk::<8>().ok_or(RuntimeError::StackUnderflowError)?;
// 		let val = u64::from_le_bytes(*slc);
// 		self.truncate(self.len() - slc.len());
// 		Ok(val)
// 	}

// 	fn pop_i64(&mut self) -> Result<i64, RuntimeError> {
// 		Ok(self.pop_u64()? as i64)
// 	}

// 	fn pop_f64(&mut self) -> Result<f64, RuntimeError> {
// 		Ok(f64::from_bits(self.pop_u64()?))
// 	}

// 	fn pop_usize(&mut self) -> Result<usize, RuntimeError> {
// 		let slc = self.last_chunk::<{std::mem::size_of::<usize>()}>().ok_or(RuntimeError::StackUnderflowError)?;
// 		let val = usize::from_le_bytes(*slc);
// 		self.truncate(self.len() - slc.len());
// 		Ok(val)
// 	}

// 	fn pop_strptr(&mut self) -> Result<Rc<str>, RuntimeError> {
// 		let ptr = self.pop_usize()? as *const u8;
// 		let len = self.pop_usize()?;
// 		let slc = unsafe { slice::from_raw_parts(ptr, len) };
// 		let str = std::str::from_utf8(slc).map_err(|_| RuntimeError::Utf8Error(slc.to_vec()))?;
// 		let rc = unsafe { Rc::from_raw(str as *const str) };
// 		Ok(rc)
// 	}

// 	fn pop_fnptr(&mut self) -> Result<Rc<str>, RuntimeError> {
// 		self.pop_strptr()
// 	}

// 	fn push_bool(&mut self, val: bool) -> Result<(), RuntimeError> {
// 		self.push(val as u8);
// 		Ok(())
// 	}

// 	fn push_u64(&mut self, val: u64) -> Result<(), RuntimeError> {
// 		let bytes = val.to_le_bytes();
// 		self.extend_from_slice(&bytes);
// 		Ok(())
// 	}

// 	fn push_i64(&mut self, val: i64) -> Result<(), RuntimeError> {
// 		self.push_u64(val as u64)
// 	}

// 	fn push_f64(&mut self, val: f64) -> Result<(), RuntimeError> {
// 		self.push_u64(val.to_bits())
// 	}

// 	fn push_usize(&mut self, val: usize) -> Result<(), RuntimeError> {
// 		let bytes = val.to_le_bytes();
// 		self.extend_from_slice(&bytes);
// 		Ok(())
// 	}

// 	fn push_strptr(&mut self, val: Rc<str>) -> Result<(), RuntimeError> {
// 		let str = Rc::into_raw(val);
// 		let ptr = str as *const u8;
// 		let len = unsafe { &*str }.len();
// 		self.push_usize(len)?;
// 		self.push_usize(ptr as usize)?;
// 		Ok(())
// 	}

// 	fn push_fnptr(&mut self, val: Rc<str>) -> Result<(), RuntimeError> {
// 		self.push_strptr(val)
// 	}

// 	fn push_lit(&mut self, val: &Literal) -> Result<(), RuntimeError> {
// 		match val {
// 			Literal::U64(val) => self.push_u64(*val),
// 			Literal::I64(val) => self.push_i64(*val),
// 			Literal::F64(val) => self.push_f64(*val),
// 			Literal::Bool(val) => self.push_bool(*val),
// 			Literal::String(val) => self.push_strptr(val.clone().into()),
// 			Literal::FnPtr(val) => self.push_fnptr(val.clone().into()),
// 			_ => todo!()
// 		}
// 	}
// }