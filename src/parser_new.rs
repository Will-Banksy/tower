mod scanner;

use std::rc::Rc;

use scanner::Scanner;
use unicode_xid::UnicodeXID;

use crate::{error::{SyntaxError, SyntaxErrorKind}, parser::{ASTNode, AnnotatedASTNode}};

// NOTE: Do I really need to rewrite both the lexer and parser completely?
//       Or should I just do the following:
//           Extend the lexer for more token types and update identifier lexing to use same rules as Rust, and add good error reporting.
//           And rewrite the parser into a recursive descent parser?
//           I could always use the scanner pattern for the lexer

pub enum TokenType {
	Identifier,
	LCurlyParen,
	RCurlyParen,
	Whitespace,
	Literal
}

fn module(scanner: &mut Scanner) -> Result<bool, SyntaxError> {
	todo!()
}

fn function(scanner: &mut Scanner) -> Result<ASTNode<AnnotatedASTNode>, SyntaxError> {
	todo!()
}

fn block(scanner: &mut Scanner) -> Result<ASTNode<AnnotatedASTNode>, SyntaxError> {
	todo!()
}

fn identifier(scanner: &mut Scanner) -> Result<Rc<str>, SyntaxError> {
	let first = scanner.take_if(|c| {
		UnicodeXID::is_xid_start(c) || c == '_'
	}).ok_or(SyntaxError::new(SyntaxErrorKind::Expected(vec![TokenType::Identifier]), scanner.cursor()))?;

	let mut ident = scanner.take_until(|c| UnicodeXID::is_xid_continue(c));

	ident.insert(0, first);

	Ok(ident.into())
}

fn literal(scanner: &mut Scanner) -> Result<ASTNode<AnnotatedASTNode>, SyntaxError> {
	todo!()
}

fn s(scanner: &mut Scanner) -> bool {
	scanner.take_of([
		' ',
		'\n',
		'\t',
		'\r',
	].iter()).is_some()
}