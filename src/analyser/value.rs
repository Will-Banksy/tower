pub struct Value {
	ty: Type,
	raw_val: im::Vector<u8>
}

pub enum ValueInner {
	Bytes(im::Vector<u8>),
	Reference {
		to: Rc<Value>
	}
}