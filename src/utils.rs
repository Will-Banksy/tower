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

pub trait IntoResult {
	fn into_result<R, E>(self, ok: R, err: E) -> Result<R, E>;
}

impl IntoResult for bool {
	fn into_result<R, E>(self, ok: R, err: E) -> Result<R, E> {
		match self {
			true => {
				Ok(ok)
			}
			false => {
				Err(err)
			}
		}
	}
}

pub trait IntoOption: Sized {
	fn into_option(self) -> Option<Self>;
}

impl<T> IntoOption for T {
	/// Wraps self in Some
	fn into_option(self) -> Option<Self> {
		Some(self)
	}
}