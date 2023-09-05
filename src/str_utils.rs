pub trait IsWhitespace {
	fn is_whitespace(&self) -> bool;
}

impl IsWhitespace for &str {
	fn is_whitespace(&self) -> bool {
		self.trim().len() == 0
	}
}