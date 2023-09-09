use tower::{lexer::tokenise, parser::parse_tokens, interpreter::interp};

fn main() {
	let towercode = include_str!("../main.tower");
	let tokens = tokenise(towercode).unwrap();
	// println!("TOKENS: {:?}", tokens);

	let ast = parse_tokens(tokens);
	println!("\n\nABSTRACT SYNTAX TREE: {:?}", ast);

	println!("\n\nINTERPRETER STARTING...");
	interp(ast).expect("Interpreter encountered an error");
}
