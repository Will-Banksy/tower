pub struct TypedTreeNode {
	file_path: String,
	cursor: usize,
	tree: TypedTree
}

pub enum TypedTree {
	Module {
		name: String,
		elems: im::Vector<TypedTreeNode>
	},
	Function {
		name: String,
		effect: StackEffect,
		body: im::Vector<TypedTreeNode>,
	},
	Type(Type),
	Word(String),
	Literal {
		ty: Type,
		value: Value
	}
}