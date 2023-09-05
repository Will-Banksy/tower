use unicode_segmentation::UnicodeSegmentation;

use crate::str_utils::IsWhitespace;

#[derive(Debug, Clone)]
pub enum Token {
	Keyword(KeywordType), // fn, if, ifelse,
	Identifier(String),
	Literal(Literal),
	FnOpen, // {
	FnClose // }
}

#[derive(Debug, PartialEq, Clone)]
pub enum KeywordType {
	Fn,
	FnDef // =
}

#[derive(Debug, Clone)]
pub enum Literal {
	U64(u64),
	I64(i64),
	F64(f64),
	Bool(bool),
	String(String),
	FnPtr(String)
}

pub struct ParsedToken {
	token: Token,
	// start_idx: usize,
	len: usize,
}

impl ParsedToken {
	pub fn new(token: Token, len: usize) -> Self {
		ParsedToken { token, len }
	}
}

pub fn tokenise(code: &str) -> Vec<Token> {
	let mut tokens = Vec::new();

	// let words: Vec<&str> = code.split_whitespace().collect();

	let code_chars: Vec<&str> = code.graphemes(true).collect();

	let mut i: usize = 0;

	let mut prev_ws = true;

	while i < code_chars.len() {
		if code_chars[i].is_whitespace() {
			i += 1;
			prev_ws = true;
		} else if code_chars[i] == "#" {
			let eol_idx = code_chars[i..].iter().position(|gc| *gc == "\n");
			if let Some(eol_idx) = eol_idx {
				i += eol_idx;
			} else {
				i = code_chars.len()
			}
			prev_ws = true;
		} else if prev_ws {
			let parsed_token = parse_token(&code_chars, i);
			if let Some(ptok) = parsed_token {
				tokens.push(ptok.token);
				i += ptok.len;
			} else {
				i += 1;
			}
			prev_ws = false;
		} else {
			i += 1;
			prev_ws = false;
		}
	}

	tokens
}

fn parse_token(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	let tok = parse_keyword(code_gc, idx);
	if tok.is_some() {
		return tok;
	}

	let tok = parse_literal(code_gc, idx);
	if tok.is_some() {
		return tok;
	}

	let tok = parse_fndelims(code_gc, idx);
	if tok.is_some() {
		return tok;
	}

	let tok = parse_identifier(code_gc, idx);
	if tok.is_some() {
		return tok;
	}

	None
}

fn parse_keyword(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	let (code, skip_amt) = get_until_whitespace(code_gc, idx);

	if code == "fn" {
		Some(ParsedToken::new(Token::Keyword(KeywordType::Fn), skip_amt))
	} else if code == "=" {
		Some(ParsedToken::new(Token::Keyword(KeywordType::FnDef), skip_amt))
	} else {
		None
	}
}

fn parse_literal(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	let (code, skip_amt) = get_until_whitespace(code_gc, idx);

	// println!("Skip amt: {}, idx: {}", skip_amt, idx);

	let last_char = code_gc[idx + skip_amt - 1];

	if last_char != "f" && last_char != "i" {
		let lit_u64 = {
			if last_char == "u" {
				code[..code.len() - 1].parse::<u64>()
			} else {
				code.parse::<u64>()
			}
		};
		if let Ok(lit_val) = lit_u64 {
			return Some(ParsedToken::new(Token::Literal(Literal::U64(lit_val)), skip_amt))
		}
	}

	if last_char != "f" {
		let lit_i64 = {
			if last_char == "i" {
				code[..code.len() - 1].parse::<i64>()
			} else {
				code.parse::<i64>()
			}
		};
		if let Ok(lit_val) = lit_i64 {
			return Some(ParsedToken::new(Token::Literal(Literal::I64(lit_val)), skip_amt))
		}
	}

	let lit_f64 = {
		if last_char == "f" {
			code[..code.len() - 1].parse::<f64>()
		} else {
			code.parse::<f64>()
		}
	};
	if let Ok(lit_val) = lit_f64 {
		return Some(ParsedToken::new(Token::Literal(Literal::F64(lit_val)), skip_amt))
	}

	let lit_bool = code.parse::<bool>();
	if let Ok(lit_val) = lit_bool {
		return Some(ParsedToken::new(Token::Literal(Literal::Bool(lit_val)), skip_amt))
	}

	let lit_str = parse_str_literal(code_gc, idx);
	if lit_str.is_some() {
		return lit_str;
	}

	None
}

fn parse_str_literal(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	if code_gc[idx] == "\"" {
		let mut num_backslashes_before = 0;
		let mut str_end_idx = 0;
		for i in (idx + 1)..code_gc.len() {
			if code_gc[i] == "\\" {
				num_backslashes_before += 1;
			} else if code_gc[i] == "\"" && num_backslashes_before % 2 == 0 { // not escaped
				str_end_idx = i;
				break;
			} else {
				num_backslashes_before = 0;
			}
		}

		let parsed_str = code_gc[(idx + 1)..str_end_idx].join("");

		// println!("Parsed string: \"{}\"", parsed_str);

		return Some(
			ParsedToken::new(
				Token::Literal(
					Literal::String(parsed_str)
				),
				(str_end_idx - idx) + 1
			)
		);
	}

	None
}

fn parse_identifier(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	// TODO: Decide on restrictions for identifiers... for now I'm just gonna accept anything
	let (code, skip_amt) = get_until_whitespace(code_gc, idx);

	Some(ParsedToken::new(Token::Identifier(code), skip_amt))
}

fn parse_fndelims(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	let (code, _) = get_until_whitespace(code_gc, idx);

	if code == "{" {
		Some(ParsedToken::new(Token::FnOpen, 1))
	} else if code == "}" {
		Some(ParsedToken::new(Token::FnClose, 1))
	} else {
		None
	}
}

fn get_until_whitespace(code_gc: &[&str], idx: usize) -> (String, usize) {
	let ws_idx = code_gc[idx..].iter().position(|gc| gc.is_whitespace());
	if let Some(ws_idx) = ws_idx {
		let ws_idx = ws_idx + idx;
		(code_gc[idx..ws_idx].join(""), ws_idx - idx)
	} else {
		(code_gc[idx..].join(""), code_gc.len() - idx)
	}
}