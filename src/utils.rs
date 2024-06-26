pub trait IsWhitespace {
	fn is_whitespace(&self) -> bool;
}

impl IsWhitespace for &str {
	fn is_whitespace(&self) -> bool {
		self.trim().len() == 0
	}
}

pub trait IncrementMut {
	fn increment(&mut self) -> Self;
}

impl IncrementMut for u64 {
	fn increment(&mut self) -> Self {
		*self += 1;
		*self
	}
}