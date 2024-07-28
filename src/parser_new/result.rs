use ScanResult::{Valid, WithErr, Unrecognised};

#[must_use]
pub enum ScanResult<T, E> {
	/// Denotes a correctly scanned/parsed structure
	Valid(T),
	/// Denotes a structure that was present but was not correctly scanned/parsed - i.e., it had an error
	WithErr(E),
	/// Denotes that the text was not recognised to be the structure, so when choosing between multiple it is likely to be other structures
	Unrecognised
}

impl<T, E> ScanResult<T, E> {
	pub fn is_valid(&self) -> bool {
		match self {
			Self::Valid(_) => true,
			_ => false
		}
	}

	pub fn require(self, err: E) -> Self {
		match self {
			Self::Unrecognised => Self::WithErr(err),
			s => s
		}
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ScanResult<U, E> {
		match self {
			Valid(v) => Valid(f(v)),
			WithErr(e) => WithErr(e),
			Unrecognised => Unrecognised
		}
	}
}

impl<T, E> From<Option<T>> for ScanResult<T, E> {
	fn from(value: Option<T>) -> Self {
		match value {
			Some(v) => Self::Valid(v),
			None => Self::Unrecognised
		}
	}
}

impl<E> From<bool> for ScanResult<(), E> {
	fn from(value: bool) -> Self {
		match value {
			true => Valid(()),
			false => Unrecognised
		}
	}
}