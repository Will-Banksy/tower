use core::fmt;
use std::collections::HashMap;

use crate::{lexer::{Literal, Token, KeywordType}, interpreter::StackItem};

pub type Instruction = Box<dyn Fn(&mut Vec<StackItem>, &HashMap<String, ASTNode>) -> Result<(), String>>;

pub enum ASTNode {
	Program(HashMap<String, ASTNode>),
	Function(Vec<ASTNode>),
	Keyword(KeywordType),
	Literal(Literal),
	Word(String),
	Instruction(Instruction)
}

impl fmt::Debug for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program(arg0) => f.debug_tuple("Program").field(arg0).finish(),
            Self::Function(arg0) => f.debug_tuple("Function").field(arg0).finish(),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::Literal(arg0) => f.debug_tuple("Literal").field(arg0).finish(),
            Self::Word(arg0) => f.debug_tuple("Word").field(arg0).finish(),
            Self::Instruction(_) => f.debug_tuple("Instruction").finish(),
        }
    }
}

pub fn parse_tokens(tokens: Vec<Token>) -> ASTNode {
	let mut fns: HashMap<String, ASTNode> = HashMap::new();

	let mut i = 0;
	while i < tokens.len() {
		if let Token::Keyword(kw_type) = &tokens[i] {
			if *kw_type == KeywordType::Fn {
				if let Some(Token::Identifier(fn_name)) = tokens.get(i + 1) {
					if let Some(Token::Keyword(KeywordType::FnDef)) = tokens.get(i + 2) {
						let fn_body: Vec<ASTNode> = {
							let mut body_toks = Vec::new();
							i += 3;
							while i < tokens.len() {
								if let Token::Keyword(KeywordType::Fn) = tokens[i] {
									i -= 1;
									break;
								}
								body_toks.push(tokens[i].clone());
								i += 1;
							}

							define_anon_fns(&mut fns, &mut body_toks);

							body_toks.into_iter().filter_map(|tok| {
								token_to_node(tok)
							}).collect()
						};
						fns.insert(fn_name.to_string(), ASTNode::Function(fn_body));
					}
				}
			}
		}
		i += 1;
	}

	ASTNode::Program(fns)
}

fn define_anon_fns(fns: &mut HashMap<String, ASTNode>, tokens: &mut Vec<Token>) {
	let mut i = 0;
	while i < tokens.len() {
		if let Token::FnOpen = tokens[i] {
			if let Some(fnclose_idx) = get_matching_fnclose(&tokens, i) {
				let mut anon_fn_body_toks: Vec<Token> = tokens.drain(i..fnclose_idx).collect();
				let anon_fn_name = format!("anon_{}", uuid::Uuid::new_v4());

				define_anon_fns(fns, &mut anon_fn_body_toks);

				let anon_fn_body = anon_fn_body_toks.into_iter().filter_map(|tok| token_to_node(tok)).collect();

				fns.insert(anon_fn_name.clone(), ASTNode::Function(anon_fn_body));
				tokens.insert(i, Token::Literal(Literal::FnPtr(anon_fn_name)))
			}
		}
		i += 1;
	}
}

fn get_matching_fnclose(tokens: &Vec<Token>, idx: usize) -> Option<usize> {
	let mut depth = 1;
	for i in (idx + 1)..tokens.len() {
		match tokens[i] {
			Token::FnOpen => depth += 1,
			Token::FnClose => {
				depth -= 1;
				if depth == 0 {
					return Some(i);
				}
			},
			_ => ()
		}
	}
	None
}

fn token_to_node(token: Token) -> Option<ASTNode> {
	if let Token::Literal(lit_val) = token {
		Some(ASTNode::Literal(lit_val))
	} else if let Token::Identifier(ident_val) = token {
		Some(ASTNode::Word(ident_val))
	} else if let Token::Keyword(kw_val) = token {
		Some(ASTNode::Keyword(kw_val))
	} else {
		None
	}
}