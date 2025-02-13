use tower::{analyser::{self, tree::{TypedTree, TypedTreeNode}}, compiler, interpreter::{builtin::builtin_functions, interp}, parser::{self, result::ScanResult, scanner::Scanner, tree::{ParseTree, ParseTreeNode}}};

fn main() {
	// compiler::compile_test_program();
	// return;

	let towercode = include_str!("../compilerdev.tower");
	// let tokens = tokenise(towercode).unwrap();
	// println!("TOKENS: {:?}", tokens);

	let mut scanner = Scanner::new(&towercode, "compilerdev.tower");
	let parse_tree = match parser::parse(&mut scanner) {
		ScanResult::Valid(tree) => tree,
		ScanResult::WithErr(e) => {
			e.print_error(&scanner, scanner.file_path(), std::io::stderr()).unwrap();
			return;
		}
		ScanResult::Unrecognised => {
			eprintln!("No module recognised");
			return;
		}
	};
	let builtin_words = builtin_functions();
	let typed_tree = match analyser::analyse(&parse_tree, &builtin_words) {
		ScanResult::Valid(tree) => tree,
		ScanResult::WithErr(e) => {
			e.print_error(&scanner, scanner.file_path(), std::io::stderr()).unwrap();
			return;
		}
		ScanResult::Unrecognised => {
			eprintln!("Unrecognised parse tree");
			return;
		}
	};

	println!("\n=== PARSE TREE ===\n");

	println!("{}", dump_parse_tree(&parse_tree, 0));

	println!("\n=== TYPED TREE ===\n");

	println!("{}", dump_typed_tree(&typed_tree, 0));

	println!("\n=== STARTING INTERPRETER ===\n");

	let stack = match interp(&typed_tree, &builtin_words) {
		Ok(stack) => stack,
		Err(e) => {
			e.print_error(&scanner, "main.tower", std::io::stderr()).unwrap();
			return;
		}
	};

	println!("\n=== POST-INTERP INFO ===\n");

	println!("Stack len: {}", stack.len());

	println!("Stack: [{}]", stack.iter().map(|v| format!("{v}")).collect::<Vec<String>>().join(", "));

	println!("\n=== LLVM OUTPUT ===\n");

	compiler::compile(typed_tree);

	// let effects = analyse(&mut ast, &mut node_id).unwrap();

	// println!("\n\nABSTRACT SYNTAX TREE: {:#?}", ast);
	// println!("\n\nSTACK EFFECTS: {:?}", effects);

	// println!("\n\nDETAILED AST: {}", print_detailed_ast(&ast, &effects, 0));

	// if let ASTNode::Module(tles) = &ast {
	// 	let fn_name = "main";
	// 	if let Some(inspected_fn) = tles.get(fn_name) {
	// 		println!("\n\nSTACK EFFECT FOR FUNCTION: {}: {}", fn_name, stack_effect_for(tles, inspected_fn).unwrap());
	// 	}
	// }

	// let effects = analyse(&mut ast);
	// println!("\n\nSTACK EFFECTS: {:?}", effects);

	// println!("\n\nINTERPRETER STARTING...");
	// let stack = Vec::<u8>::new();
	// if let Err(e) = interp(ast, &mut (Box::new(stack) as Box<dyn TowerStack>)) {
	// 	eprintln!("Runtime Error: {e:?}");
	// }
}

fn dump_parse_tree(tree: &ParseTreeNode, depth: u32) -> String { // TODO: depth is not used - Use it or remove it
	match &tree.tree {
		ParseTree::Module { name, elems } => format!("Module(name: {name}, elems: [\n{}])", elems.iter().map(|(elem_name, elem)| format!("\t{elem_name}: {},\n", dump_parse_tree(elem, depth + 1))).collect::<String>()),
		ParseTree::Function { name, body } => format!("Function(name: {name}, body: [\n{}\t])", body.iter().map(|node| format!("\t\t{},\n", dump_parse_tree(node, depth + 1))).collect::<String>()),
		ParseTree::Literal(lit) => format!("Literal({lit:?})"),
		ParseTree::Identifier(word) => format!("Identifier({word})"),
		ParseTree::Struct { name, fields } => format!("Struct(name: {name}, fields: [\n{}\t])", fields.iter().map(|(fname, ftype)| format!("\t\t{fname}: {ftype},\n")).collect::<String>()),
		ParseTree::Enum { name, fields } => format!("Struct(name: {name}, [\n{}\t])", fields.iter().map(|(fname, ftype)| format!("\t\t{fname} {ftype},\n")).collect::<String>()),
		ParseTree::Constructor(ty) => format!("Constructor(of: {ty})"),
		ParseTree::FieldAccess(ident) => format!("FieldAccess(field: {ident})")
	}
}

fn dump_typed_tree(tree: &TypedTreeNode, depth: u32) -> String {
	match &tree.tree {
		TypedTree::Module { name, elems } => format!("Module(name: {name}, elems: [\n{}])", elems.iter().map(|(elem_name, elem)| format!("\t{elem_name}: {},\n", dump_typed_tree(elem, depth + 1))).collect::<String>()),
		TypedTree::Function { name, effect, body } => format!("Function(name: {name}, effect: {effect}, body: [\n{}\t])", body.iter().map(|node| format!("\t\t{},\n", dump_typed_tree(node, depth + 1))).collect::<String>()),
		TypedTree::Type(ty) => format!("Type({ty})"),
		TypedTree::Word(word) => format!("Word({word})"),
		TypedTree::BuiltinWord(word) => format!("BuiltinWord({word})"),
		TypedTree::Literal { ty, value } => format!("Literal(type: {ty}, value: (unable to be displayed))"),
		TypedTree::Constructor { ty, effect } => format!("Constructor(of: {ty}, effect: {effect})"),
		TypedTree::FieldAccess { name } => format!("FieldAccess(field: {name})")
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