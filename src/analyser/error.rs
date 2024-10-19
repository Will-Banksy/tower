use std::{fmt::Display, io::{self, Write}};

use crate::parser::scanner::Scanner;

use super::ttype::Type;

#[derive(Clone)]
pub struct AnalysisError {
	kind: AnalysisErrorKind,
	cursor: usize,
}

impl AnalysisError {
	pub fn new(kind: AnalysisErrorKind, cursor: usize) -> Self {
		AnalysisError {
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

		writeln!(writer, "Syntax Error at {file_name}:{col}:{row} - {self}")?;
		writeln!(writer, "{} | ", " ".repeat(row_str.len()))?;
		writeln!(writer, "{row} | {}", context.replace("\t", "    "))?;
		writeln!(writer, "{cursor_indicator}")?;

		Ok(())
	}
}

impl Display for AnalysisError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			AnalysisErrorKind::IncompatibleTypes { source, dest } => {
				write!(f, "source type {source} is incompatible with dest type {dest}")
			}
			AnalysisErrorKind::TypeIsNotFunction { tname } => {
				write!(f, "expected function instead of type name {tname}")
			}
			AnalysisErrorKind::FunctionIsNotType { fname } => {
				write!(f, "expected type name instead of function {fname}")
			}
			AnalysisErrorKind::NoSuchFunction { fname } => {
				write!(f, "function {fname} was not found in scope")
			}
			AnalysisErrorKind::NoSuchType { tname } => {
				write!(f, "type {tname} was not found in scope")
			}
			AnalysisErrorKind::UnconstructableType { tname } => {
				write!(f, "type {tname} cannot be constructed (is not a struct or enum variant)")
			}
		}
	}
}

#[derive(Clone)]
pub enum AnalysisErrorKind {
	IncompatibleTypes {
		source: Type,
		dest: Type
	},
	TypeIsNotFunction {
		tname: String,
	},
	FunctionIsNotType {
		fname: String,
	},
	NoSuchFunction {
		fname: String,
	},
	NoSuchType {
		tname: String,
	},
	UnconstructableType {
		tname: String,
	}
}