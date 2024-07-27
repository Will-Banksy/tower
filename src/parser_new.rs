pub mod scanner;

use std::{collections::HashMap, rc::Rc, sync::RwLock};

use scanner::Scanner;
use unicode_xid::UnicodeXID;

use crate::{error::{SyntaxError, SyntaxErrorKind}, lexer::{Literal, Token}, parser::{ASTNode, AnnotatedASTNode, NodeId}, utils::{IntoOption, IntoResult}};

// NOTE: Do I really need to rewrite both the lexer and parser completely?
//       Or should I just do the following:
//           Extend the lexer for more token types and update identifier lexing to use same rules as Rust, and add good error reporting.
//           And rewrite the parser into a recursive descent parser?
//           I could always use the scanner pattern for the lexer

// TODO: Current idea: Yeah just rewrite the lexer and parser in one using the scanner pattern. Why not

// TODO: Need to probably examine all errors and aggregate them - e.g. if choosing between A and B for C, we're gonna get "Expected A" and "Expected B" separately - ideally we'd have "Expected A, B"
// TODO: Indeed, error handling needs a lot of work - We're currently basically ignoring all errors
//           We perhaps, in various scanner functions, need some way to "commit to a path" - i.e. if we're looking for any functions, and we come across "fn"
//           We perhaps need some enum other than Result that holds either Ok, Err, or a secret third thing: Skip
//           NOTE: Doing the above with Option<Result<_, SyntaxError>> where None is Skip

type ParseResult<R> = Option<Result<R, SyntaxError>>;

static NODE_ID: RwLock<NodeId> = RwLock::new(NodeId::new());

pub enum TokenType {
	Identifier,
	LCurlyParen,
	RCurlyParen,
	Whitespace,
	Literal,
	KeywordFn,
	Quote,
	EscapeSequence
}

pub fn parse(scanner: &mut Scanner, node_id: &mut NodeId) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	*NODE_ID.write().unwrap() = *node_id;

	let ret = Ok(
		module(scanner)?
	);

	*node_id = *NODE_ID.read().unwrap();

	ret
}

/// Returns a Module ASTNode
fn module(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("module");

	scanner.take_any(s);

	let nodes = scanner.take_any(function);

	let tles = nodes.into_iter().map(|(fn_name, fn_body)| (fn_name.to_string(), fn_body.annotated(NODE_ID.write().unwrap().inc()))).collect();

	scanner.take_any(s);

	eprintln!("module end");

	Ok(ASTNode::Module(tles, "main".to_string()))
}

/// Returns a Function ASTNode, paired with the function name
fn function<'a>(scanner: &'a mut Scanner) -> ParseResult<(String, ASTNode<AnnotatedASTNode>)> {
	eprintln!("function");

	if !scanner.take_str("fn") {
		return Err(SyntaxError::expected(vec![TokenType::KeywordFn], scanner.cursor()));
	}

	scanner.take_some(s);

	let fn_name = if let ASTNode::Identifier(s) = identifier(scanner)? {
		s
	} else {
		unimplemented!()
	};

	scanner.take_some(s);

	let fn_body = block(scanner)?;

	eprintln!("function end");

	Ok((
		fn_name.to_string(),
		ASTNode::Function(Box::new(fn_body.annotated(NODE_ID.write().unwrap().inc())))
	))
}

/// Returns a Block ASTNode
fn block(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("block");

	scanner.take('{').into_result((), SyntaxError::expected(vec![TokenType::LCurlyParen], scanner.cursor()))?;

	let nodes = scanner.take_any(|scanner| -> ParseResult<ASTNode<AnnotatedASTNode>> {
		scanner.take_any(s);

		let ret = match scanner.take_choice(vec![
			Box::new(identifier),
			Box::new(literal)
		]) {
			Some(r) => Ok(r),
			None => {
				Err(SyntaxError::expected(vec![TokenType::Identifier, TokenType::Literal], scanner.cursor()))
			}
		};

		scanner.take_any(s);

		ret
	});

	scanner.take_any(s);

	eprintln!("block cursor: {:?} at {}", scanner.peek(), scanner.cursor());

	scanner.take('}').into_result((), SyntaxError::expected(vec![TokenType::LCurlyParen], scanner.cursor()))?;

	eprintln!("block end");

	Ok(
		ASTNode::Block(nodes.into_iter().map(|node| node.annotated(NODE_ID.write().unwrap().inc())).collect())
	)
}

/// Returns an Identifier ASTNode
fn identifier(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("identifier");

	let first = scanner.take_if(|c| {
		UnicodeXID::is_xid_start(c) || c == '_'
	}).ok_or(SyntaxError::expected(vec![TokenType::Identifier], scanner.cursor()))?;

	let mut ident = scanner.take_until(|c| UnicodeXID::is_xid_continue(c));

	ident.insert(0, first);

	eprintln!("identifier end");

	Ok(ASTNode::Identifier(ident.into()))
}

/// Returns a Literal ASTNode
fn literal(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("literal");

	let ret = scanner.take_choice(vec![
		Box::new(literal_string),
		Box::new(literal_integer),
		Box::new(literal_float)
	]).map(|lit| ASTNode::Literal(lit)).ok_or(SyntaxError::expected(vec![TokenType::Literal], scanner.cursor()))?;

	eprintln!("literal end");

	Ok(ret)
}

/// Returns a Literal
fn literal_string(scanner: &mut Scanner) -> ParseResult<Literal> {
	eprintln!("literal_string");

	eprintln!("literal_string cursor: {:?} at {}", scanner.peek(), scanner.cursor());

	if !scanner.take('\"') {
		return None;
	};

	eprintln!("literal_string #1");

	// TODO: Have a think about how to handle syntax errors here...

	let chars = scanner.take_any::<char, SyntaxError>(|scanner| {
		scanner.take_choice::<char, SyntaxError>(vec![
			Box::new(|scanner| {
				eprintln!("literal_string take \\");
				if !scanner.take('\\') {
					return None;
				}

				match scanner.take_choice::<char, SyntaxError>(vec![
					Box::new(|scanner| {
						match scanner.take_of([
							'\\',
							'n',
							't',
							'r',
							'0',
							'"',
						].iter()) {
							Some(c) => {
								Some(Ok(match c {
									'\\' => '\\',
									'n' => '\n',
									't' => '\t',
									'r' => '\r',
									'0' => '\0',
									'"' => '"',
									_ => unimplemented!()
								}))
							}
							None => {
								None// Err(SyntaxError::expected(vec![TokenType::EscapeSequence], scanner.cursor()))
							}
						}
					}),
					Box::new(|scanner| {
						if !scanner.take('x') {
							return None;
						}

						let mut sb = String::new();

						for _ in 0..2 {
							match scanner.take_if(|c| {
								c.is_ascii_hexdigit()
							}) {
								Some(c) => { sb.push(c); },
								None => { return Some(Err(SyntaxError::expected(vec![TokenType::EscapeSequence], scanner.cursor()))); }
							};
						}

						let hex_val = u32::from_str_radix(&sb.trim(), 16).unwrap();

						Some(Ok(
							unsafe { char::from_u32_unchecked(hex_val) } // NOTE: This may cause issues
						))
					})
				]) {
					None => {
						Some(Err(SyntaxError::expected(vec![TokenType::EscapeSequence], scanner.cursor())))
					}
					r => r
				}
			}),
			Box::new(|scanner| {
				eprintln!("literal_string take CHAR");
				match scanner.pop() {
					Some(c) => {
						if c == '\"' {
							None
						} else {
							Some(Ok(c))
						}
					}
					None => Some(Err(SyntaxError::expected(vec![TokenType::Quote], scanner.cursor())))
				}
			})
		])
	});

	let mut sb = String::new();

	for c in chars {
		match c {
			Ok(c) => {
				sb.push(c);
			}
			Err(e) => {
				return Some(Err(e));
			}
		}
	}

	scanner.advance(1);

	eprintln!("literal_string end");

	Some(Ok(Literal::String(sb)))
}

/// Returns a Literal
fn literal_integer(scanner: &mut Scanner) -> ParseResult<Literal> { // TODO
	None

	// let negative = scanner.take('-');

	// scanner.take_choice(vec![
	// 	Box::new(|scanner| {
	// 		scanner.take_str("0b").into_result((), SyntaxError::empty(scanner.cursor()))
	// 	})
	// ]);

	// todo!()
}

/// Returns a Literal
fn literal_float(scanner: &mut Scanner) -> ParseResult<Literal> { // TODO
	None
}

fn s(scanner: &mut Scanner) -> ParseResult<()> {
	scanner.take_of([
		' ',
		'\n',
		'\t',
		'\r',
	].iter()).map(|_| ()).ok_or(SyntaxError::expected(vec![TokenType::Whitespace], scanner.cursor())).into_option()
}