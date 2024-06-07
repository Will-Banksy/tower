use unicode_segmentation::UnicodeSegmentation;

use crate::utils::IsWhitespace;

// TODO: Better parsing method: https://depth-first.com/articles/2021/12/16/a-beginners-guide-to-parsing-in-rust/

#[derive(Debug, Clone)]
pub enum Token {
	Keyword(KeywordType),
	Identifier(String),
	Literal(Literal),
}

#[derive(Debug, PartialEq, Clone)]
pub enum KeywordType {
	Fn, // fn
	FnDef, // =
	FnEnd, // ;
	AnonFnOpen, // {
	AnonFnClose, // }
	ReorderBefore, // <- // NOTE: This will be syntax sugar for more natural usage of ifs, ifelses, etc. e.g. "if <- <conditional> <if-true>" vs <conditional> <if-true> if" // TODO
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

pub fn tokenise(code: &str) -> Result<Vec<Token>, String> {
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
			let parsed_token = parse_token(&code_chars, i)?;
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

	Ok(tokens)
}

fn parse_token(code_gc: &[&str], idx: usize) -> Result<Option<ParsedToken>, String> {
	let tok = parse_keyword(code_gc, idx);
	if tok.is_some() {
		return Ok(tok);
	}

	let tok = parse_literal(code_gc, idx)?;
	if tok.is_some() {
		return Ok(tok);
	}

	let tok = parse_identifier(code_gc, idx);
	if tok.is_some() {
		return Ok(tok);
	}

	Ok(None)
}

fn parse_keyword(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	let (code, skip_amt) = get_until_whitespace(code_gc, idx);

	if code == "fn" {
		Some(ParsedToken::new(Token::Keyword(KeywordType::Fn), skip_amt))
	} else if code == "=" {
		Some(ParsedToken::new(Token::Keyword(KeywordType::FnDef), skip_amt))
	} else if code == ";" {
		Some(ParsedToken::new(Token::Keyword(KeywordType::FnEnd), skip_amt))
	} else if code == "{" {
		Some(ParsedToken::new(Token::Keyword(KeywordType::AnonFnOpen), skip_amt))
	} else if code == "}" {
		Some(ParsedToken::new(Token::Keyword(KeywordType::AnonFnClose), skip_amt))
	} else {
		None
	}
}

fn parse_literal(code_gc: &[&str], idx: usize) -> Result<Option<ParsedToken>, String> {
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
			return Ok(Some(ParsedToken::new(Token::Literal(Literal::U64(lit_val)), skip_amt)))
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
			return Ok(Some(ParsedToken::new(Token::Literal(Literal::I64(lit_val)), skip_amt)))
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
		return Ok(Some(ParsedToken::new(Token::Literal(Literal::F64(lit_val)), skip_amt)))
	}

	let lit_bool = code.parse::<bool>();
	if let Ok(lit_val) = lit_bool {
		return Ok(Some(ParsedToken::new(Token::Literal(Literal::Bool(lit_val)), skip_amt)))
	}

	let lit_str = parse_str_literal(code_gc, idx)?;
	if lit_str.is_some() {
		return Ok(lit_str);
	}

	Ok(None)
}

fn parse_str_literal(code_gc: &[&str], idx: usize) -> Result<Option<ParsedToken>, String> {
	if code_gc[idx] == "\"" {
		let mut sb = String::new();

		let mut i = idx + 1;
		let mut escaped = false;
		while i < code_gc.len() {
			if escaped {
				escaped = false;
				if code_gc[i] == "\"" {
					sb.push('"');
				} else if code_gc[i] == "\\" {
					sb.push('\\');
				} else if code_gc[i] == "n" {
					sb.push('\n');
				} else if code_gc[i] == "t" {
					sb.push('\t');
				} else if code_gc[i] == "r" {
					sb.push('\r');
				} else if code_gc[i] == "0" {
					sb.push('\0');
				} else if code_gc[i] == "u" {
					if let Some(&"{") = code_gc.get(i + 1) {
						i += 2;
						let mut u_hex_chars = 0;
						while i < code_gc.len() {
							if code_gc[i] == "}" {
								break;
							} else {
								u_hex_chars += 1;
								i += 1;
							}
						}
						let u_start = i - u_hex_chars;
						let hex_str = code_gc[u_start..i].join("");

						let invalid_code_pt_err = Err(format!("[ERROR]: Value {} is not a valid unicode code point", hex_str));
						if u_hex_chars > 6 {
							return invalid_code_pt_err;
						}

						let hex_val = u32::from_str_radix(&hex_str.trim(), 16).map_err(|e| format!("[ERROR]: Failed to convert hex string {} to u32: {}", hex_str, e))?;
						if let Some(code_point) = char::from_u32(hex_val) {
							sb.push(code_point)
						} else {
							return invalid_code_pt_err;
						}
					} else {
						return Err("[ERROR]: Expected { in unicode escape \\u{...}".into())
					}
				}
			} else if code_gc[i] == "\\" {
				escaped = true;
			} else if code_gc[i] == "\"" {
				break;
			} else {
				sb.push_str(code_gc[i])
			}
			i += 1;
		}

		return Ok(Some(ParsedToken::new(Token::Literal(Literal::String(sb)), i - idx)))
	}

	Ok(None)
}

fn parse_identifier(code_gc: &[&str], idx: usize) -> Option<ParsedToken> {
	// TODO: Decide on restrictions for identifiers... for now I'm just gonna accept anything
	let (code, skip_amt) = get_until_whitespace(code_gc, idx);

	if code_gc[idx] == "&" {
		Some(ParsedToken::new(Token::Literal(Literal::FnPtr(code_gc[(idx + 1)..(idx + skip_amt)].join(""))), skip_amt))
	} else {
		Some(ParsedToken::new(Token::Identifier(code), skip_amt))
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