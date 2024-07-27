use std::io::{self, Write};

use crate::parser_new::{scanner::Scanner, TokenType};

pub struct SyntaxError {
	kind: SyntaxErrorKind,
	cursor: usize,
}

impl SyntaxError {
	pub fn new(kind: SyntaxErrorKind, cursor: usize) -> Self {
		SyntaxError {
			kind,
			cursor
		}
	}

	/// Syntax sugar for `SyntaxError::new(SyntaxErrorKind::Expected(types), cursor)`
	pub fn expected(types: Vec<TokenType>, cursor: usize) -> Self {
		SyntaxError::new(SyntaxErrorKind::Expected(types), cursor)
	}

	pub fn empty(cursor: usize) -> Self {
		SyntaxError::new(SyntaxErrorKind::None, cursor)
	}

	/// Pretty-prints the error, including context retrieved from the scanner
	pub fn print_error(&self, scanner: &Scanner, file_name: &str, mut writer: impl Write) -> Result<(), io::Error> {
		let context = scanner.get_context(self.cursor);
		let (col, row) = scanner.get_col_row(self.cursor);
		let cursor_indicator = [ " ".repeat(row), "^".to_string() ].join("");

		writeln!(writer, "Syntax Error at {file_name}:{col}:{row}")?;
		writeln!(writer, "{context}")?;
		writeln!(writer, "{cursor_indicator}")?;

		Ok(())
	}
}

pub enum SyntaxErrorKind {
	None,
	Expected(Vec<TokenType>),
}

#[derive(Debug)]
pub enum RuntimeError {
	StackUnderflowError,
	Utf8Error(Vec<u8>),
	FunctionMissingError(String),
	ModuleNotFoundError
}