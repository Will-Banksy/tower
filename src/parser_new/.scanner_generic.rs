pub struct Scanner<'a, T> where T: Clone {
	content: &'a [T],
	cursor: usize
}

impl<'a, T> Scanner<'a, T> where T: Clone + PartialEq {
	pub fn new(content: &'a [T]) -> Self {
		Scanner {
			content,
			cursor: 0
		}
	}

	pub fn cursor(&self) -> usize {
		self.cursor
	}

	pub fn has_next(&self) -> bool {
		self.cursor < self.content.len()
	}

	pub fn peek(&self) -> Option<T> {
		self.content.last().cloned()
	}

	pub fn advance(&mut self, count: usize) {
		self.cursor += count;
		if self.cursor >= self.content.len() {
			self.cursor = self.content.len();
		}
	}

	pub fn pop(&mut self) -> Option<T> {
		let ret = self.peek();
		self.advance(1);
		ret
	}

	pub fn take(&mut self, c: T) -> bool {
		match self.content.get(self.cursor) {
			Some(ref n) if **n == c => {
				self.pop();
				true
			}
			_ => false
		}
	}

	pub fn take_str(&mut self, s: &[T]) -> bool {
		match &self.content.get(self.cursor..s.len()) {
			Some(n) if *n == s => {
				self.advance(s.len());
				true
			}
			_ => false
		}
	}

	/// If the next character matches any of the characters returned by the iterator, then it returns the matching character, otherwise None
	pub fn take_of<'c>(&mut self, i: impl Iterator<Item = &'c T>) -> Option<T> where T: 'c {
		for c in i {
			if match self.peek() {
				Some(ref n) if *n == *c => {
					self.pop();
					true
				}
				_ => false
			} {
				return Some(c.clone())
			}
		}
		None
	}

	/// If there is a character available and pred returns true, returns that character, advancing the cursor. Otherwise, returns None
	pub fn take_if(&mut self, pred: impl FnOnce(T) -> bool) -> Option<T> {
		if self.has_next() && pred(self.peek().unwrap()) {
			self.pop()
		} else {
			None
		}
	}

	/// Builds a string of every character `pred` returns true on, advancing the cursor, until the first time pred returns false or until the end of the available content
	pub fn take_until(&mut self, pred: impl Fn(T) -> bool) -> Vec<T> {
		let mut sb = Vec::new();

		while self.has_next() && pred(self.peek().unwrap()) {
			sb.push(self.pop().unwrap());
		}

		sb
	}
}