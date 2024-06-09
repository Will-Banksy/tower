pub struct Scanner<'a> {
	content: &'a str,
	content_chars: Vec<char>,
	cursor: usize
}

impl<'a> Scanner<'a> {
	pub fn new(content: &'a str) -> Self {
		Scanner {
			content,
			content_chars: content.chars().collect(),
			cursor: 0
		}
	}

	pub fn cursor(&self) -> usize {
		self.cursor
	}

	pub fn has_next(&self) -> bool {
		self.cursor < self.content_chars.len()
	}

	pub fn peek(&self) -> Option<char> {
		self.content_chars.last().copied()
	}

	pub fn advance(&mut self, count: usize) {
		self.cursor += count;
		if self.cursor >= self.content_chars.len() {
			self.cursor = self.content_chars.len();
		}
	}

	pub fn pop(&mut self) -> Option<char> {
		let ret = self.peek();
		self.advance(1);
		ret
	}

	pub fn take(&mut self, c: char) -> bool {
		match self.content_chars.get(self.cursor) {
			Some(&n) if n == c => {
				self.pop();
				true
			}
			_ => false
		}
	}

	pub fn take_str(&mut self, s: &str) -> bool {
		match &self.content.get(self.cursor..s.len()) {
			Some(n) if *n == s => {
				self.advance(s.len());
				true
			}
			_ => false
		}
	}

	/// If the next character matches any of the characters returned by the iterator, then it returns the matching character, otherwise None
	pub fn take_of<'c>(&mut self, i: impl Iterator<Item = &'c char>) -> Option<char> {
		for &c in i {
			if match self.peek() {
				Some(n) if n == c => {
					self.pop();
					true
				}
				_ => false
			} {
				return Some(c)
			}
		}
		None
	}

	/// If there is a character available and pred returns true, returns that character, advancing the cursor. Otherwise, returns None
	pub fn take_if(&mut self, pred: impl FnOnce(char) -> bool) -> Option<char> {
		if self.has_next() && pred(self.peek().unwrap()) {
			self.pop()
		} else {
			None
		}
	}

	/// Builds a string of every character `pred` returns true on, advancing the cursor, until the first time pred returns false or until the end of the available content
	pub fn take_until(&mut self, pred: impl Fn(char) -> bool) -> String {
		let mut sb = String::new();

		while self.has_next() && pred(self.peek().unwrap()) {
			sb.push(self.pop().unwrap());
		}

		sb
	}
}