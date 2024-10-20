use std::{fmt::Display, io::{self, Write}};

use crate::parser::scanner::Scanner;

pub struct RuntimeError {
	kind: RuntimeErrorKind,
	cursor: usize
}

impl RuntimeError {
	pub fn new(kind: RuntimeErrorKind, cursor: usize) -> RuntimeError {
		RuntimeError {
			kind,
			cursor
		}
	}

	/// Pretty-prints the error, including context retrieved from the scanner
	pub fn print_error(&self, scanner: &Scanner, file_name: &str, mut writer: impl Write) -> Result<(), io::Error> {
		// BUG: Alignment is off when there are multi-byte or multi code point characters such as ✨ in the context line before the cursor

		let context = scanner.get_context(self.cursor);
		let (col, row) = scanner.get_col_row(self.cursor);
		let row_str = format!("{row}");
		let num_tabs = context.chars().filter(|&c| c == '\t').count();
		let cursor_indicator = [ " ".repeat(row_str.len()), " | ".to_string(), "    ".repeat(num_tabs), " ".repeat(col.saturating_sub(1 + num_tabs)), "^".to_string() ].join("");

		writeln!(writer, "Runtime Error at {file_name}:{col}:{row} - {self}")?;
		writeln!(writer, "{} | ", " ".repeat(row_str.len()))?;
		writeln!(writer, "{row} | {}", context.replace("\t", "    "))?;
		writeln!(writer, "{cursor_indicator}")?;

		Ok(())
	}
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			RuntimeErrorKind::StackUnderflowError => {
				write!(f, "stack underflow error - attempted to pop from empty stack")
			},
			RuntimeErrorKind::Utf8Error(vec) => {
				write!(f, "error decoding UTF-8 string from bytes {vec:?}")
			},
			RuntimeErrorKind::FunctionMissingError(name) => {
				write!(f, "function {name} not found")
			},
			RuntimeErrorKind::ModuleNotFoundError => {
				write!(f, "no module found")
			},
		}
	}
}

#[derive(Debug)]
pub enum RuntimeErrorKind {
	StackUnderflowError,
	Utf8Error(Vec<u8>),
	FunctionMissingError(String),
	ModuleNotFoundError
}