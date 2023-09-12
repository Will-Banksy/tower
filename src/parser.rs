use core::fmt;
use std::collections::HashMap;

use crate::{lexer::{Literal, Token, KeywordType}, interpreter::StackItem};

pub type Instruction = Box<dyn Fn(&mut Vec<StackItem>, &HashMap<String, ASTNode>) -> Result<(), String>>;

pub enum ASTNode {
	Module(HashMap<String, ASTNode>),
	Function(Box<ASTNode>),
	Keyword(KeywordType),
	Literal(Literal),
	Word(String),
	Instruction(Instruction),
	Block(Vec<ASTNode>),
}

impl fmt::Debug for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Module(arg0) => f.debug_tuple("Program").field(arg0).finish(),
            Self::Function(arg0) => f.debug_tuple("Function").field(arg0).finish(),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::Literal(arg0) => f.debug_tuple("Literal").field(arg0).finish(),
            Self::Word(arg0) => f.debug_tuple("Word").field(arg0).finish(),
            Self::Instruction(_) => f.debug_tuple("Instruction").finish(),
            Self::Block(arg0) => f.debug_tuple("Block").field(arg0).finish(),
        }
    }
}

pub fn parse_tokens(tokens: Vec<Token>) -> ASTNode { // TODO: Develop a system for making errors aware of the context
	// Top level elements
	let mut tles: HashMap<String, ASTNode> = HashMap::new();

	let mut i = 0;
	while i < tokens.len() {
		if let Token::Keyword(KeywordType::Fn) = &tokens[i] {
			if let Some(Token::Identifier(fn_name)) = tokens.get(i + 1) {
				if let Some(Token::Keyword(KeywordType::FnDef)) = tokens.get(i + 2) {
					let fn_body: Vec<ASTNode> = {
						let mut body_toks = Vec::new();
						i += 3;
						while i < tokens.len() {
							if let Token::Keyword(KeywordType::FnEnd) = tokens[i] {
								break;
							}
							body_toks.push(tokens[i].clone());
							i += 1;
						}

						define_anon_fns(&mut tles, &mut body_toks);

						body_toks.into_iter().filter_map(|tok| {
							token_to_node(tok)
						}).collect()
					};
					tles.insert(fn_name.to_string(), ASTNode::Function(Box::new(ASTNode::Block(fn_body))));
				}
			}
		}
		i += 1;
	}

	ASTNode::Module(tles)
}

fn define_anon_fns(tles: &mut HashMap<String, ASTNode>, tokens: &mut Vec<Token>) {
	let mut i = 0;
	while i < tokens.len() {
		if let Token::Keyword(KeywordType::AnonFnOpen) = tokens[i] {
			if let Some(fnclose_idx) = get_matching_fnclose(&tokens, i) {
				let mut anon_fn_body_toks: Vec<Token> = tokens.drain(i..=fnclose_idx).collect();
				anon_fn_body_toks.remove(0);
				anon_fn_body_toks.remove(anon_fn_body_toks.len() - 1);
				let anon_fn_name = format!("anon_{}", uuid::Uuid::new_v4());

				define_anon_fns(tles, &mut anon_fn_body_toks);

				let anon_fn_body = anon_fn_body_toks.into_iter().filter_map(|tok| token_to_node(tok)).collect();

				tles.insert(anon_fn_name.clone(), ASTNode::Function(Box::new(ASTNode::Block(anon_fn_body))));
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
			Token::Keyword(KeywordType::AnonFnOpen) => depth += 1,
			Token::Keyword(KeywordType::AnonFnClose) => {
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