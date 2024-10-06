#[derive(Debug, Clone, PartialEq)]
pub enum ParseTreeType {
	None,
	Module,
	Function,
	Struct,
	Enum,
	Identifier,
	Literal
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTreeNode {
	pub file_path: String,
	pub cursor: usize,
	pub tree: ParseTree
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseTree {
	Module {
		name: String,
		elems: im::HashMap<String, ParseTreeNode>
	},
	Function {
		name: String,
		body: im::Vector<ParseTreeNode>
	},
	Struct {
		name: String,
		fields: im::HashMap<String, String>
	},
	Enum {
		name: String,
		fields: im::HashMap<String, String>
	},
	Identifier(String),
	Literal(Literal),
}

impl ParseTree {
	pub fn wrap(self, file_path: impl Into<String>, cursor: usize) -> ParseTreeNode {
		ParseTreeNode {
			file_path: file_path.into(),
			cursor,
			tree: self
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
	U128(u128),
	U64(u64),
	U32(u32),
	U16(u16),
	U8(u8),
	I128(i128),
	I64(i64),
	I32(i32),
	I16(i16),
	I8(i8),
	F64(f64),
	F32(f64),
	Bool(bool),
	String(String),
	FnPtr(String)
}