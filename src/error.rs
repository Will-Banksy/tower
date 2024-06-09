use crate::parser_new::TokenType;

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
}

pub enum SyntaxErrorKind {
	Expected(Vec<TokenType>)
}

#[derive(Debug)]
pub enum RuntimeError {
	StackUnderflowError,
	Utf8Error(Vec<u8>),
	FunctionMissingError(String),
	ModuleNotFoundError
}