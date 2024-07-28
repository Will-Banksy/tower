pub mod scanner;
pub mod result;

use std::sync::RwLock;

use result::ScanResult::{self, Valid, WithErr, Unrecognised};
use scanner::Scanner;
use unicode_xid::UnicodeXID;

use crate::{brk, error::{SyntaxError, SyntaxErrorKind}, lexer::Literal, parser::{ASTNode, ASTNodeType, AnnotatedASTNode, NodeId}};

type ParseResult<T> = ScanResult<T, SyntaxError>;// Option<Result<T, SyntaxError>>;

static NODE_ID: RwLock<NodeId> = RwLock::new(NodeId::new());

#[derive(Debug)]
pub enum TokenType {
	None,
	Identifier,
	LCurlyParen,
	RCurlyParen,
	Whitespace,
	Literal,
	KeywordFn,
	Quote,
	EscapeSequence,
	Block
}

pub fn parse(scanner: &mut Scanner, node_id: &mut NodeId) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	*NODE_ID.write().unwrap() = *node_id;

	let ret = brk!(module(scanner));

	scanner.take_any(s);

	if scanner.has_next() {
		return WithErr(SyntaxError::new(SyntaxErrorKind::Unexpected, ASTNodeType::Module, scanner.cursor()));
	}

	*node_id = *NODE_ID.read().unwrap();

	Valid(ret)
}

/// Returns a Module ASTNode
fn module(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("module");

	let (nodes, err) = scanner.take_any(|scanner| {
		scanner.take_any(s);
		let fun = brk!(function(scanner));

		Valid(fun)
	});
	if let Some(e) = err {
		return WithErr(e);
	}

	let tles = nodes.into_iter().map(|(fn_name, fn_body)| (fn_name.to_string(), fn_body.annotated(NODE_ID.write().unwrap().inc()))).collect();

	eprintln!("module end");

	Valid(ASTNode::Module(tles, "main".to_string()))
}

/// Returns a Function ASTNode, paired with the function name
fn function<'a>(scanner: &'a mut Scanner) -> ParseResult<(String, ASTNode<AnnotatedASTNode>)> {
	eprintln!("function");

	brk!(ParseResult::from(scanner.take_str("fn")));

	brk!(ParseResult::from(scanner.take_some(s)).require(SyntaxError::expected(vec![TokenType::Whitespace], ASTNodeType::Function, scanner.cursor())));//.ok_or(SyntaxError::expected(vec![TokenType::Whitespace], ASTNodeType::Function, scanner.cursor()));

	let fn_name = match identifier(scanner).require(SyntaxError::expected(vec![TokenType::Identifier], ASTNodeType::Function, scanner.cursor())) {
		Valid(ASTNode::Identifier(s)) => s,
		WithErr(e) => {
			return WithErr(e);
		}
		Valid(_) => unreachable!(),
		_ => unreachable!()
	};

	brk!(ParseResult::from(scanner.take_some(s)).require(SyntaxError::expected(vec![TokenType::Whitespace], ASTNodeType::Function, scanner.cursor())));

	let fn_body = brk!(block(scanner).require(SyntaxError::expected(vec![TokenType::Block], ASTNodeType::Function, scanner.cursor())));//.ok_or(SyntaxError::expected(vec![TokenType::Block], scanner.cursor()))?;

	eprintln!("function end");

	Valid((
		fn_name.to_string(),
		ASTNode::Function(Box::new(fn_body.annotated(NODE_ID.write().unwrap().inc())))
	))
}

/// Returns a Block ASTNode
fn block(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("block");

	brk!(scanner.take('{').into());

	let (nodes, err) = scanner.take_any(|scanner| -> ParseResult<ASTNode<AnnotatedASTNode>> {
		scanner.take_any(s);

		let ret = brk!(scanner.take_choice(vec![
			Box::new(identifier),
			Box::new(literal)
		]));

		scanner.take_any(s);

		Valid(ret)
	});
	if let Some(e) = err {
		return WithErr(e);
	}

	scanner.take_any(s);

	brk!(ParseResult::from(scanner.take('}')).require(SyntaxError::expected(vec![TokenType::Identifier, TokenType::Literal, TokenType::RCurlyParen], ASTNodeType::Block, scanner.cursor())));

	eprintln!("block end");

	Valid(
		ASTNode::Block(nodes.into_iter().map(|node| node.annotated(NODE_ID.write().unwrap().inc())).collect())
	)
}

/// Returns an Identifier ASTNode
fn identifier(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("identifier");

	let first = brk!(scanner.take_if(|c| {
		UnicodeXID::is_xid_start(c) || c == '_'
	}).into());

	let mut ident = scanner.take_until(|c| UnicodeXID::is_xid_continue(c));

	ident.insert(0, first);

	eprintln!("identifier end");

	Valid(ASTNode::Identifier(ident.into()))
}

/// Returns a Literal ASTNode
fn literal(scanner: &mut Scanner) -> ParseResult<ASTNode<AnnotatedASTNode>> {
	eprintln!("literal");

	let ret = brk!(scanner.take_choice(vec![
		Box::new(literal_string),
		Box::new(literal_integer),
		Box::new(literal_float)
	]).map(|lit| ASTNode::Literal(lit)));

	eprintln!("literal end");

	Valid(ret)
}

/// Returns a Literal
fn literal_string(scanner: &mut Scanner) -> ParseResult<Literal> {
	eprintln!("literal_string");

	if !scanner.take('\"') {
		return Unrecognised;
	};

	eprintln!("literal_string #1");

	let (chars, err) = scanner.take_any::<char, SyntaxError>(|scanner| {
		scanner.take_choice::<char, SyntaxError>(vec![
			Box::new(|scanner| {
				if !scanner.take('\\') {
					return Unrecognised;
				}

				scanner.take_choice::<char, SyntaxError>(vec![
					Box::new(|scanner| {
						ParseResult::from(scanner.take_of([
							'\\',
							'n',
							't',
							'r',
							'0',
							'"',
						].iter())).map(|c| match c {
							'\\' => '\\',
							'n' => '\n',
							't' => '\t',
							'r' => '\r',
							'0' => '\0',
							'"' => '"',
							_ => unimplemented!()
						})
					}),
					Box::new(|scanner| {
						brk!(scanner.take('x').into());

						let mut sb = String::new();

						for _ in 0..2 {
							sb.push(brk!(ParseResult::from(scanner.take_if(|c| {
								c.is_ascii_hexdigit()
							})).require(SyntaxError::expected(vec![TokenType::EscapeSequence], ASTNodeType::Literal, scanner.cursor()))))
						}

						let hex_val = u32::from_str_radix(&sb.trim(), 16).unwrap();

						Valid(
							unsafe { char::from_u32_unchecked(hex_val) } // NOTE: This may cause issues
						)
					})
				]).require(SyntaxError::expected(vec![TokenType::EscapeSequence], ASTNodeType::Literal, scanner.cursor()))
			}),
			Box::new(|scanner| {
				match scanner.pop() {
					Some(c) => {
						if c == '\"' {
							Unrecognised
						} else {
							Valid(c)
						}
					}
					None => {
						WithErr(SyntaxError::expected(vec![TokenType::Quote], ASTNodeType::Literal, scanner.cursor()))
					}
				}
			})
		])
	});
	if let Some(e) = err {
		return WithErr(e);
	}

	let string = chars.into_iter().collect();

	scanner.advance(1);

	eprintln!("literal_string end @ {}", scanner.cursor());

	Valid(Literal::String(string))
}

/// Returns a Literal
fn literal_integer(scanner: &mut Scanner) -> ParseResult<Literal> { // TODO
	Unrecognised

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
	Unrecognised
}

fn s(scanner: &mut Scanner) -> ParseResult<()> {
	scanner.take_of([
		' ',
		'\n',
		'\t',
		'\r',
	].iter()).map(|_| ()).into()
}