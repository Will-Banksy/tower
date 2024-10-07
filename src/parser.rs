pub mod scanner;
pub mod result;
pub mod tree;

use std::num::IntErrorKind;

use result::ScanResult::{self, Valid, WithErr, Unrecognised};
use scanner::Scanner;
use tree::{ParseTree, ParseTreeNode, ParseTreeType, Literal};
use unicode_xid::UnicodeXID;

use crate::{analyser::TowerType, brk, error::{SyntaxError, SyntaxErrorKind}};

type ParseResult<T> = ScanResult<T, SyntaxError>;// Option<Result<T, SyntaxError>>;

#[derive(Debug, Clone)]
pub enum TokenType { // TODO: Evaluate these, and ideally have these represented in the grammar
	None,
	Identifier,
	LCurlyParen,
	RCurlyParen,
	Whitespace,
	Literal,
	Number,
	KeywordFn,
	Quote,
	EscapeSequence,
	Block,
	Colon,
	ConstructorArrow
}

pub fn parse(scanner: &mut Scanner) -> ParseResult<ParseTreeNode> {
	let cursor = scanner.cursor();

	let ret = brk!(module(scanner));

	scanner.take_any(s);

	if scanner.has_next() {
		return WithErr(SyntaxError::new(SyntaxErrorKind::Unexpected, ParseTreeType::Module, scanner.cursor()));
	}

	Valid(ret.wrap(scanner.file_path(), cursor))
}

/// Returns a Module ASTNode
fn module(scanner: &mut Scanner) -> ParseResult<ParseTree> {
	eprintln!("module");

	let (nodes, err) = scanner.take_any(|scanner| {
		scanner.take_any(s);

		scanner.take_choice(vec![
			Box::new(|scanner| {
				let cursor = scanner.cursor();
				let (name, body) = brk!(function(scanner));

				Valid((name, body.wrap(scanner.file_path(), cursor)))
			}),
			Box::new(|scanner| {
				let cursor = scanner.cursor();
				let (name, structure) = brk!(structure(scanner));

				Valid((name, structure.wrap(scanner.file_path(), cursor)))
			})
		])
	});
	if let Some(e) = err {
		return WithErr(e);
	}

	let elems = nodes.into_iter().collect();

	eprintln!("module end");

	Valid(ParseTree::Module { name: "".into(), elems })
}

/// Returns a Function ASTNode, paired with the function name
fn function(scanner: &mut Scanner) -> ParseResult<(String, ParseTree)> {
	eprintln!("function");

	brk!(ParseResult::from(scanner.take_str("fn")));

	brk!(ParseResult::from(scanner.take_some(s)).require(SyntaxError::expected(vec![TokenType::Whitespace], ParseTreeType::Function, scanner.cursor())));//.ok_or(SyntaxError::expected(vec![TokenType::Whitespace], ASTNodeType::Function, scanner.cursor()));

	let fn_name = match identifier(scanner).require(SyntaxError::expected(vec![TokenType::Identifier], ParseTreeType::Function, scanner.cursor())) {
		Valid(ParseTree::Identifier(s)) => s,
		WithErr(e) => {
			return WithErr(e);
		}
		_ => unreachable!()
	};

	scanner.take_any(s);

	let fn_body = brk!(block(scanner).require(SyntaxError::expected(vec![TokenType::Block], ParseTreeType::Function, scanner.cursor())));//.ok_or(SyntaxError::expected(vec![TokenType::Block], scanner.cursor()))?;

	eprintln!("function end");

	Valid((
		fn_name.to_string(),
		ParseTree::Function {
			name: fn_name.to_string(),
			body: fn_body
		}
	))
}

fn structure(scanner: &mut Scanner) -> ParseResult<(String, ParseTree)> {
	eprintln!("struct");

	brk!(scanner.take_str("struct").into());

	brk!(ParseResult::from(scanner.take_some(s)).require(SyntaxError::expected(vec![TokenType::Whitespace], ParseTreeType::Struct, scanner.cursor())));

	let name = match brk!(identifier(scanner).require(SyntaxError::expected(vec![TokenType::Identifier], ParseTreeType::Struct, scanner.cursor()))) {
		ParseTree::Identifier(s) => s,
		_ => unreachable!()
	};

	scanner.take_any(s);

	brk!(ParseResult::from(scanner.take('{')).require(SyntaxError::expected(vec![TokenType::LCurlyParen], ParseTreeType::Struct, scanner.cursor())));

	let (fields, err) = scanner.take_any::<(String, String), SyntaxError>(|scanner| {
		scanner.take_any(s);

		let field_name = match brk!(identifier(scanner)) {
			ParseTree::Identifier(s) => s,
			_ => unreachable!()
		};

		scanner.take_any(s);

		brk!(ParseResult::from(scanner.take(':')).require(SyntaxError::expected(vec![TokenType::Colon], ParseTreeType::Struct, scanner.cursor())));

		scanner.take_any(s);

		let field_type = match brk!(ParseResult::from(identifier(scanner)).require(SyntaxError::expected(vec![TokenType::Identifier], ParseTreeType::Struct, scanner.cursor()))) {
			ParseTree::Identifier(s) => s,
			_ => unreachable!()
		};

		Valid((field_name, field_type))
	});
	if let Some(e) = err {
		return WithErr(e);
	}

	scanner.take_any(s);

	brk!(ParseResult::from(scanner.take('}')).require(SyntaxError::expected(vec![TokenType::RCurlyParen], ParseTreeType::Struct, scanner.cursor())));

	let fields: im::OrdMap<String, String> = fields.into_iter().collect();

	eprintln!("struct end");

	Valid((
		name.clone(),
		ParseTree::Struct { name, fields }
	))
}

/// Returns a Block ASTNode
fn block(scanner: &mut Scanner) -> ParseResult<im::Vector<ParseTreeNode>> {
	eprintln!("block");

	brk!(scanner.take('{').into());

	let (nodes, err) = scanner.take_any(|scanner| -> ParseResult<ParseTreeNode> {
		scanner.take_any(s);

		let cursor = scanner.cursor();

		let ret = brk!(scanner.take_choice(vec![
			Box::new(identifier),
			Box::new(literal),
			Box::new(constructor_struct)
		]));

		scanner.take_any(s);

		Valid(ret.wrap(scanner.file_path(), cursor))
	});
	if let Some(e) = err {
		return WithErr(e);
	}

	scanner.take_any(s);

	brk!(ParseResult::from(scanner.take('}')).require(SyntaxError::expected(vec![TokenType::Identifier, TokenType::Literal, TokenType::RCurlyParen], ParseTreeType::Function, scanner.cursor())));

	eprintln!("block end");

	Valid(
		nodes.into_iter().collect()
	)
}

/// Returns an Identifier ASTNode
fn identifier(scanner: &mut Scanner) -> ParseResult<ParseTree> {
	eprintln!("identifier");

	let first = brk!(scanner.take_if(|c| {
		UnicodeXID::is_xid_start(c) || c == '_'
	}).into());

	let mut ident = scanner.take_until(|c| UnicodeXID::is_xid_continue(c));

	ident.insert(0, first);

	eprintln!("identifier end");

	Valid(ParseTree::Identifier(ident.into()))
}

/// Returns a Literal ASTNode
fn literal(scanner: &mut Scanner) -> ParseResult<ParseTree> {
	eprintln!("literal");

	let ret = brk!(scanner.take_choice(vec![
		Box::new(literal_string),
		Box::new(literal_integer),
		Box::new(literal_float),
	]).map(|lit| ParseTree::Literal(lit)));

	eprintln!("literal end");

	Valid(ret)
}

/// Returns a Literal
fn literal_string(scanner: &mut Scanner) -> ParseResult<Literal> {
	eprintln!("literal_string");

	brk!(scanner.take('\"').into());

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
							_ => unreachable!()
						})
					}),
					Box::new(|scanner| {
						brk!(scanner.take('x').into());

						let mut sb = String::new();

						for _ in 0..2 {
							sb.push(brk!(ParseResult::from(scanner.take_if(|c| {
								c.is_ascii_hexdigit()
							})).require(SyntaxError::expected(vec![TokenType::EscapeSequence], ParseTreeType::Literal, scanner.cursor()))))
						}

						let hex_val = u32::from_str_radix(&sb.trim(), 16).unwrap();

						Valid(
							unsafe { char::from_u32_unchecked(hex_val) } // FIXME: This may cause issues
						)
					})
				]).require(SyntaxError::expected(vec![TokenType::EscapeSequence], ParseTreeType::Literal, scanner.cursor()))
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
						WithErr(SyntaxError::expected(vec![TokenType::Quote], ParseTreeType::Literal, scanner.cursor()))
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
fn literal_integer(scanner: &mut Scanner) -> ParseResult<Literal> { // TODO: To allow better type inference, we could perhaps only store the literal strings for now, and then turn the strings into integers once we've decided the types
	// let i = 1000000000000000000; // Rust seems to default to i32 unless explicitly told otherwise for literals
	let negative = scanner.take('-');

	let start_of_int = scanner.cursor();

	let (num, denary) = brk!(scanner.take_choice::<(u128, bool), SyntaxError>(vec![
		Box::new(move |scanner| { // Parse binary literal
			literal_integer_radix(scanner, 2).map(|n| (n, false))
		}),
		Box::new(move |scanner| { // Parse octal literal
			literal_integer_radix(scanner, 8).map(|n| (n, false))
		}),
		Box::new(move |scanner| { // Parse hex literal
			literal_integer_radix(scanner, 16).map(|n| (n, false))
		}),
		Box::new(move |scanner| { // Parse denary literal
			literal_integer_radix(scanner, 10).map(|n| (n, true))
		})
	]));

	let start_of_suffix = scanner.cursor();

	let suffix = brk!(scanner.try_take(|scanner| {
			let signed = brk!(scanner.take_of(['u', 'i'].iter()).into()) == 'i';

			let bits = brk!(literal_integer_radix(scanner, 10).optional());

			Valid((signed, bits))
		}
	).optional());

	let num_type = match suffix {
		Some((signed, Some(bits))) => {
			if !signed && negative {
				return WithErr(SyntaxError::new(SyntaxErrorKind::NegativeUnsignedLiteral, ParseTreeType::Literal, start_of_suffix));
			}

			match bits {
				8 => { if signed { TowerType::I8 } else { TowerType::U8 } },
				16 => { if signed { TowerType::I16 } else { TowerType::U16 } },
				32 => { if signed { TowerType::I32 } else { TowerType::U32 } },
				64 => { if signed { TowerType::I64 } else { TowerType::U64 } },
				128 => { if signed { TowerType::I128 } else { TowerType::U128 } },
				_ => return WithErr(SyntaxError::new(SyntaxErrorKind::InvalidIntegerSize, ParseTreeType::Literal, start_of_suffix + 1))
			}
		}
		Some((signed, None)) => {
			if !signed && negative {
				return WithErr(SyntaxError::new(SyntaxErrorKind::NegativeUnsignedLiteral, ParseTreeType::Literal, start_of_suffix));
			}

			if signed {
				TowerType::I32
			} else {
				TowerType::U32
			}
		}
		None => {
			if !denary {
				TowerType::U32
			} else {
				TowerType::I32
			}
		}
	};

	let literal = match num_type {
		TowerType::U128 => Literal::U128(num),
		TowerType::U64 => Literal::U64(brk!(downcast_uint(num, TowerType::U64, start_of_int))),
		TowerType::U32 => Literal::U32(brk!(downcast_uint(num, TowerType::U32, start_of_int))),
		TowerType::U16 => Literal::U16(brk!(downcast_uint(num, TowerType::U16, start_of_int))),
		TowerType::U8 => Literal::U8(brk!(downcast_uint(num, TowerType::U8, start_of_int))),
		TowerType::I128 => Literal::I128(num as i128),
		TowerType::I64 => Literal::I64(brk!(downcast_uint::<u64, _>(num, TowerType::U64, start_of_int)) as i64),
		TowerType::I32 => Literal::I32(brk!(downcast_uint::<u32, _>(num, TowerType::U32, start_of_int)) as i32),
		TowerType::I16 => Literal::I16(brk!(downcast_uint::<u16, _>(num, TowerType::U16, start_of_int)) as i16),
		TowerType::I8 => Literal::I8(brk!(downcast_uint::<u8, _>(num, TowerType::U8, start_of_int)) as i8),
		_ => unreachable!()
	};

	Valid(literal)
}

fn downcast_uint<T, F>(num: F, num_type: TowerType, cursor: usize) -> ParseResult<T> where T: TryFrom<F>, F: ToString + Copy {
	match num.try_into().ok() {
		Some(n) => Valid(n),
		None => {
			WithErr(SyntaxError::new(SyntaxErrorKind::LiteralIntegerOverflow { num: num.to_string(), target_type: num_type }, ParseTreeType::Literal, cursor))
		}
	}
}

/// Parses an integer (including prefix, not including suffix) using radix (radix ∈ [2, 8, 10, 16])
fn literal_integer_radix(scanner: &mut Scanner, radix: u8) -> ParseResult<u128> {
	let start_of_int = scanner.cursor();

	let is_radix_digit = match radix {
		2 => |c: char| { c == '0' || c == '1' },
		8 => |c: char| { ['0', '1', '2', '3', '4', '5', '6', '7'].iter().find(|&&oc| oc == c).is_some() },
		10 => |c: char| { c.is_ascii_digit() },
		16 => |c: char| { c.is_ascii_hexdigit() },
		_ => unimplemented!("Illegal radix value: {radix}")
	};

	let prefix = match radix {
		2 => "0b",
		8 => "0o",
		10 => "",
		16 => "0x",
		_ => unreachable!()
	};

	let prefix_required = !prefix.is_empty();

	brk!(scanner.take_str(prefix).into());

	let result = ParseResult::from(scanner.take_some::<char, SyntaxError>(|scanner| {
		scanner.take_if(is_radix_digit).into()
	}));
	let (chars, err) = match prefix_required {
		true => brk!(result.require(SyntaxError::expected(vec![TokenType::Number], ParseTreeType::Literal, scanner.cursor()))),
		false => brk!(result)
	};
	if let Some(e) = err {
		return WithErr(e);
	}

	let num_str: String = chars.into_iter().collect();

	let num = match u128::from_str_radix(&num_str, radix as u32) {
		Ok(v) => v,
		Err(e) => {
			match e.kind() {
				IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
					return WithErr(SyntaxError::new(SyntaxErrorKind::LiteralIntegerOverflow { num: format!("{prefix}{num_str}"), target_type: TowerType::U128 }, ParseTreeType::Literal, start_of_int))
				}
				_ => unreachable!()
			}
		}
	};

	Valid(num)
}

/// Returns a Literal
fn literal_float(scanner: &mut Scanner) -> ParseResult<Literal> { // TODO
	Unrecognised
}

fn constructor_struct(scanner: &mut Scanner) -> ParseResult<ParseTree> {
	brk!(scanner.take_str("->").into());

	brk!(ParseResult::from(scanner.take_some(s)).require(SyntaxError::expected(vec![TokenType::Whitespace], ParseTreeType::Constructor, scanner.cursor())));

	let ident = match brk!(identifier(scanner).require(SyntaxError::expected(vec![TokenType::Identifier], ParseTreeType::Constructor, scanner.cursor()))) {
		ParseTree::Identifier(s) => s,
		_ => unreachable!()
	};

	Valid(ParseTree::Constructor(ident))
}

fn s(scanner: &mut Scanner) -> ParseResult<()> {
	scanner.take_if(|c| c.is_whitespace()).map(|_| ()).into()
}