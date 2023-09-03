use tower::lexer::tokenise;

fn main() {
	let towercode = include_str!("../main.tower");
	let tokens = tokenise(towercode);
	println!("{:?}", tokens)
}
