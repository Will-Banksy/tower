use tower::{lexer::tokenise, parser::{parse_tokens, ASTNode}, interpreter::interp, analyser::stack_effect_for};

fn main() {
	let towercode = include_str!("../main.tower");
	let tokens = tokenise(towercode).unwrap();
	// println!("TOKENS: {:?}", tokens);

	let ast = parse_tokens(tokens);
	println!("\n\nABSTRACT SYNTAX TREE: {:?}", ast);

	if let ASTNode::Module(tles) = &ast {
		let fn_name = "push_some_things";
		if let Some(inspected_fn) = tles.get(fn_name) {
			println!("\n\nSTACK EFFECT FOR FUNCTION: {}: {}", fn_name, stack_effect_for(tles, inspected_fn).unwrap());
		}
	}

	println!("\n\nINTERPRETER STARTING...");
	interp(ast).expect("Interpreter encountered an error");
}
