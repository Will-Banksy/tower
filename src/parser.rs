use core::fmt;
use std::{collections::HashMap, ops::Deref, rc::Rc};

use crate::{error::RuntimeError, lexer::{KeywordType, Literal, Token}, stack::TowerStack};

pub type Instruction = Rc<dyn Fn(&mut Box<dyn TowerStack>, &HashMap<String, AnnotatedASTNode>) -> Result<(), RuntimeError>>;

#[derive(Debug)]
pub enum ASTNodeType {
	None,
	Module,
	Function,
	Keyword,
	Literal,
	Identifier,
	Block
}

#[derive(Clone)]
pub enum ASTNode<N: Clone> {
	Module(HashMap<String, N>, String),
	Function(Box<N>), // Box is just so the enum isn't recursive in size
	Keyword(KeywordType),
	Literal(Literal),
	Identifier(String),
	/// NOTE: This should not be used
	Instruction(Instruction),
	Block(Vec<N>),
}

// impl<O, N> ASTNode<O> where O: Clone, N: Clone {
// 	fn into_new<F>(node: ASTNode<O>, f: F) where F: Fn(ASTNode<O>) -> ASTNode<N> {
// 		todo!()
// 	}
// }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId {
	inner: u64
}

impl NodeId {
	pub const fn new() -> Self {
		NodeId {
			inner: 0
		}
	}

	/// Increments inner value by 1 and returns copy of self
	pub fn inc(&mut self) -> Self {
		self.inner += 1;
		*self
	}
}

impl Deref for NodeId {
	type Target = u64;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[derive(Clone, Debug)]
pub struct AnnotatedASTNode {
	pub node: ASTNode<AnnotatedASTNode>,
	pub id: NodeId
}

impl AnnotatedASTNode {
	pub fn new(node: ASTNode<AnnotatedASTNode>, id: NodeId) -> Self {
		Self {
			node,
			id
		}
	}
}

// #[derive(Clone)]
// pub struct ASTNode {
// 	pub n_type: ASTNodeType,
// 	pub effect: Option<StackEffect>
// }

impl ASTNode<AnnotatedASTNode> {
	pub fn annotated(self, id: NodeId) -> AnnotatedASTNode {
		AnnotatedASTNode::new(self, id)
	}
}

impl<N> fmt::Debug for ASTNode<N> where N: Clone + fmt::Debug {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ASTNode::Module(arg0, _arg1) => f.debug_tuple("Module").field(arg0).finish(),
			ASTNode::Function(arg0) => f.debug_tuple("Function").field(arg0).finish(),
			ASTNode::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
			ASTNode::Literal(arg0) => f.debug_tuple("Literal").field(arg0).finish(),
			ASTNode::Identifier(arg0) => f.debug_tuple("Word").field(arg0).finish(),
			ASTNode::Instruction(_) => f.debug_tuple("Instruction").finish(),
			ASTNode::Block(arg0) => f.debug_tuple("Block").field(arg0).finish(),
		}
	}
}

pub fn parse_tokens(tokens: Vec<Token>, node_id: &mut NodeId) -> AnnotatedASTNode { // TODO: Develop a system for making errors aware of the context
	// Top level elements
	let mut tles: HashMap<String, AnnotatedASTNode> = HashMap::new();

	let mut id = node_id;

	let mut i = 0;
	while i < tokens.len() {
		if let Token::Keyword(KeywordType::Fn) = &tokens[i] {
			if let Some(Token::Identifier(fn_name)) = tokens.get(i + 1) {
				if let Some(Token::Keyword(KeywordType::FnDef)) = tokens.get(i + 2) {
					let fn_body: Vec<AnnotatedASTNode> = {
						let mut body_toks = Vec::new();
						i += 3;
						while i < tokens.len() {
							if let Token::Keyword(KeywordType::FnEnd) = tokens[i] {
								break;
							}
							body_toks.push(tokens[i].clone());
							i += 1;
						}

						define_anon_fns(&mut tles, &mut body_toks, &mut id);

						body_toks.into_iter().filter_map(|tok| {
							token_to_node(tok, &mut id)
						}).collect()
					};
					tles.insert(fn_name.to_string(),
						AnnotatedASTNode::new(
							ASTNode::Function(Box::new(
								AnnotatedASTNode::new(ASTNode::Block(fn_body), id.inc())
							)),
							id.inc()
						)
					);
				}
			}
		}
		i += 1;
	}

	AnnotatedASTNode::new(
		ASTNode::Module(tles, "main".into()),
		id.inc()
	)
}

fn define_anon_fns(tles: &mut HashMap<String, AnnotatedASTNode>, tokens: &mut Vec<Token>, id: &mut NodeId) {
	let mut i = 0;
	while i < tokens.len() {
		if let Token::Keyword(KeywordType::AnonFnOpen) = tokens[i] {
			if let Some(fnclose_idx) = get_matching_fnclose(&tokens, i) {
				let mut anon_fn_body_toks: Vec<Token> = tokens.drain(i..=fnclose_idx).collect();
				anon_fn_body_toks.remove(0);
				anon_fn_body_toks.remove(anon_fn_body_toks.len() - 1);
				let anon_fn_name = format!("anon_{}", uuid::Uuid::new_v4());

				define_anon_fns(tles, &mut anon_fn_body_toks, id);

				let anon_fn_body = anon_fn_body_toks.into_iter().filter_map(|tok| token_to_node(tok, id)).collect();

				tles.insert(anon_fn_name.clone(), AnnotatedASTNode::new(
					ASTNode::Function(Box::new(
						AnnotatedASTNode::new(ASTNode::Block(anon_fn_body), id.inc())
					)),
					id.inc()
				));
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

fn token_to_node(token: Token, id: &mut NodeId) -> Option<AnnotatedASTNode> {
	if let Token::Literal(lit_val) = token {
		Some(AnnotatedASTNode::new(ASTNode::Literal(lit_val), id.inc()))
	} else if let Token::Identifier(ident_val) = token {
		Some(AnnotatedASTNode::new(ASTNode::Identifier(ident_val), id.inc()))
	} else if let Token::Keyword(kw_val) = token {
		Some(AnnotatedASTNode::new(ASTNode::Keyword(kw_val), id.inc()))
	} else {
		None
	}
}