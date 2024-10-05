use std::{fmt::{self, Display}, io::{self, Write}};

use crate::{analyser::TowerType, parser_new::{scanner::Scanner, tree::ParseTreeType, TokenType}};

// TODO: Is there any better way to handle errors? Or will this do?

pub struct SyntaxError {
	kind: SyntaxErrorKind,
	cursor: usize,
	while_parsing: ParseTreeType
}

impl SyntaxError {
	pub fn new(kind: SyntaxErrorKind, while_parsing: ParseTreeType, cursor: usize) -> Self {
		SyntaxError {
			kind,
			cursor,
			while_parsing
		}
	}

	/// Syntax sugar for `SyntaxError::new(SyntaxErrorKind::Expected(types), cursor)`
	pub fn expected(types: Vec<TokenType>, while_parsing: ParseTreeType, cursor: usize) -> Self {
		SyntaxError::new(SyntaxErrorKind::Expected(types), while_parsing, cursor)
	}

	pub fn empty(cursor: usize) -> Self {
		SyntaxError::new(SyntaxErrorKind::None, ParseTreeType::None, cursor)
	}

	/// Pretty-prints the error, including context retrieved from the scanner
	pub fn print_error(&self, scanner: &Scanner, file_name: &str, mut writer: impl Write) -> Result<(), io::Error> {
		// BUG: Alignment is off when there are multi-byte or multi code point characters such as ✨ in the context line before the cursor

		let context = scanner.get_context(self.cursor);
		let (col, row) = scanner.get_col_row(self.cursor);
		let row_str = format!("{row}");
		let num_tabs = context.chars().filter(|&c| c == '\t').count();
		let cursor_indicator = [ " ".repeat(row_str.len()), " | ".to_string(), "    ".repeat(num_tabs), " ".repeat(col.saturating_sub(1 + num_tabs)), "^".to_string() ].join("");

		writeln!(writer, "Syntax Error at {file_name}:{col}:{row} - {self}")?;
		writeln!(writer, "{} | ", " ".repeat(row_str.len()))?;
		writeln!(writer, "{row} | {}", context.replace("\t", "    "))?;
		writeln!(writer, "{cursor_indicator}")?;

		Ok(())
	}
}

impl Display for SyntaxError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.kind {
			SyntaxErrorKind::Expected(tokens) => {
				write!(f, "while parsing {:?}, expected [{}]", self.while_parsing, tokens.iter().map(|tok| format!("{tok:?}")).collect::<Vec<String>>().join(", "))
			}
			SyntaxErrorKind::None => {
				write!(f, "while parsing {:?}, empty error", self.while_parsing)
			}
			SyntaxErrorKind::Unexpected => {
				write!(f, "while parsing {:?}, unexpected string", self.while_parsing)
			}
			SyntaxErrorKind::LiteralIntegerOverflow { num, target_type } => {
				write!(f, "while parsing {:?}, integer literal {} doesn't fit in target type {:?}", self.while_parsing, num, target_type)
			}
			SyntaxErrorKind::InvalidIntegerSize => {
				write!(f, "while parsing {:?}, invalid integer size", self.while_parsing)
			}
			SyntaxErrorKind::NegativeUnsignedLiteral => {
				write!(f, "while parsing {:?}, negative unsigned integer literal", self.while_parsing)
			}
		}
	}
}

pub enum SyntaxErrorKind {
	None,
	Expected(Vec<TokenType>),
	Unexpected,
	LiteralIntegerOverflow {
		num: String,
		target_type: TowerType
	},
	InvalidIntegerSize,
	NegativeUnsignedLiteral,
}

#[derive(Debug)]
pub enum RuntimeError {
	StackUnderflowError,
	Utf8Error(Vec<u8>),
	FunctionMissingError(String),
	ModuleNotFoundError
}