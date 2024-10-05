pub enum Type {
	Opaque {
		name: String,
		size: Option<usize>,
	},
	Transparent {
		name: String,
		fields: im::Vector<(String, Type)>,
		/// Whether this type is a sum type/enum (true) or product type/struct (false)
		sum_type: bool,
	},
	Reference {
		to: Box<Type>,
	}
}