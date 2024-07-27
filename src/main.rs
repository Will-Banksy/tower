use std::collections::HashMap;

use tower::{analyser::{analyse, StackEffect}, interpreter::interp, lexer::tokenise, parser::{parse_tokens, ASTNode, AnnotatedASTNode, NodeId}, parser_new::{self, scanner::Scanner}, stack::TowerStack};

// TODO: Rethink error handling. Look at resources for error handling in compilers/interpreters. Maybe use log crate, maybe don't. Need context-aware errors too

fn main() {
	let towercode = include_str!("../main.tower");
	// let tokens = tokenise(towercode).unwrap();
	// println!("TOKENS: {:?}", tokens);

	let mut node_id = NodeId::new();

	let mut scanner = Scanner::new(&towercode);
	let mut ast = match parser_new::parse(&mut scanner, &mut node_id) {
		Ok(ast) => ast,
		Err(e) => {
			e.print_error(&scanner, "main.tower", std::io::stderr()).unwrap();
			return;
		}
	}.annotated(node_id.inc());

	let effects = analyse(&mut ast, &mut node_id).unwrap();

	// println!("\n\nABSTRACT SYNTAX TREE: {:#?}", ast);
	// println!("\n\nSTACK EFFECTS: {:?}", effects);

	println!("\n\nDETAILED AST: {}", print_detailed_ast(&ast, &effects, 0));

	// if let ASTNode::Module(tles) = &ast {
	// 	let fn_name = "main";
	// 	if let Some(inspected_fn) = tles.get(fn_name) {
	// 		println!("\n\nSTACK EFFECT FOR FUNCTION: {}: {}", fn_name, stack_effect_for(tles, inspected_fn).unwrap());
	// 	}
	// }

	// let effects = analyse(&mut ast);
	// println!("\n\nSTACK EFFECTS: {:?}", effects);

	println!("\n\nINTERPRETER STARTING...");
	let stack = Vec::<u8>::new();
	if let Err(e) = interp(ast, &mut (Box::new(stack) as Box<dyn TowerStack>)) {
		eprintln!("Runtime Error: {e:?}");
	}
}

fn print_detailed_ast(ast: &AnnotatedASTNode, effects: &HashMap<NodeId, StackEffect>, depth: u32) -> String { // TODO: Refactor to actually use the depth for printing (it's a mess rn)
	match &ast.node {
		ASTNode::Module(tles, entry_point) => format!("Module(entry_point: {entry_point}, tles: [\n{}])", tles.iter().map(|(tle_name, tle)| format!("\t{tle_name}: {},\n", print_detailed_ast(tle, effects, depth + 1))).collect::<String>()),
		ASTNode::Function(node) => format!("Function({})", print_detailed_ast(node, effects, depth + 1)),
		ASTNode::Keyword(_) => unimplemented!(),
		ASTNode::Literal(lit) => format!("Literal({lit:?}, stack_effect: {})", effects.get(&ast.id).unwrap()),
		ASTNode::Identifier(word) => format!("Word({word}, stack_effect: {})", effects.get(&ast.id).unwrap()),
		ASTNode::Instruction(_) => format!("Instruction(stack_effect: {})", effects.get(&ast.id).unwrap()),
		ASTNode::Block(nodes) => format!("Block(nodes: [\n{}\t], stack_effect: {})", nodes.iter().map(|node| format!("\t\t{},\n", print_detailed_ast(node, effects, depth + 1))).collect::<String>(), effects.get(&ast.id).unwrap()),
	}
}

// impl<N> fmt::Debug for ASTNode<N> where N: Clone + fmt::Debug {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             ASTNode::Module(arg0, _arg1) => f.debug_tuple("Module").field(arg0).finish(),
//             ASTNode::Function(arg0) => f.debug_tuple("Function").field(arg0).finish(),
//             ASTNode::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
//             ASTNode::Literal(arg0) => f.debug_tuple("Literal").field(arg0).finish(),
//             ASTNode::Word(arg0) => f.debug_tuple("Word").field(arg0).finish(),
//             ASTNode::Instruction(_) => f.debug_tuple("Instruction").finish(),
//             ASTNode::Block(arg0) => f.debug_tuple("Block").field(arg0).finish(),
//         }
//     }
// }