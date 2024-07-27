use crate::{error::SyntaxError, parser::{ASTNode, AnnotatedASTNode}};

pub struct Scanner<'a> {
	content: &'a str,
	content_chars: Vec<char>,
	cursor: usize
}

// enum ScanResult<R, E> {
// 	Some(Result<R, E>),
// 	Skip
// }

impl<'a> Scanner<'a> { // TODO: Introduce a better naming scheme, with separation of functions operating on the underlying data and higher order functions
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

	/// Returns the line pointed to by the cursor
	pub fn get_context(&self, cursor: usize) -> &'a str {
		// Find the previous newline
		let start = self.content[0..=cursor].rfind(|c| c == '\n').unwrap_or(0);

		// Find the next newline or carriage return (as that comes before newline on windows)
		let end = cursor + self.content[cursor..].find(|c| c == '\n' || c == '\r').unwrap_or(self.content.len() - cursor);

		// The reason I operated on bytes rather than chars in this method is cause I can't slice with char indices
		&self.content[start..end]
	}

	pub fn get_col_row(&self, cursor: usize) -> (usize, usize) {
		let row = self.content_chars[0..=cursor].iter().filter(|&&c| c == '\n').count();
		let col = self.content_chars[0..=cursor].iter().fold::<usize, _>(0, |acc, &c| {
			if c == '\n' {
				0
			} else {
				acc + 1
			}
		});

		(col, row)
	}

	pub fn has_next(&self) -> bool {
		self.cursor < self.content_chars.len()
	}

	pub fn peek(&self) -> Option<char> {
		self.content_chars.get(self.cursor).copied()
	}

	pub fn advance(&mut self, count: usize) {
		self.cursor += count;
		if self.cursor >= self.content_chars.len() {
			self.cursor = self.content_chars.len();
		}
	}

	/// Returns the next character if there is one, and advances the cursor
	pub fn pop(&mut self) -> Option<char> {
		let ret = self.peek();
		self.advance(1);
		ret
	}

	/// If the next character matches the passed-in character, return true
	pub fn take(&mut self, c: char) -> bool {
		match self.content_chars.get(self.cursor) {
			Some(&n) if n == c => {
				self.pop();
				true
			}
			_ => false
		}
	}

	/// If the passed-in string matches, return true
	pub fn take_str(&mut self, s: &str) -> bool {
		match &self.content_chars.get(self.cursor..s.len()) {
			Some(n) => {
				if n.iter().collect::<String>() == s {
					self.advance(s.len());
					true
				} else {
					false
				}
			}
			_ => false
		}
	}

	/// If the next character matches any of the characters returned by the iterator, then it returns the matching character, otherwise None
	pub fn take_of<'c>(&mut self, i: impl Iterator<Item = &'c char>) -> Option<char> {
		for &c in i {
			if self.take(c) {
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

	/// Attempts to match with the given function, returning with the function return value, and not advancing the cursor in case of a None
	pub fn try_take<R, E>(&mut self, f: impl FnOnce(&mut Self) -> Option<Result<R, E>>) -> Option<Result<R, E>> {
		let cursor = self.cursor();

		match f(self) {
			Some(r) => Some(r),
			None => {
				self.cursor = cursor;
				None
			}
		}
	}

	/// Attempts to match with the given function any number of times, returning a list of all return values
	pub fn take_any<R, E>(&mut self, mut f: impl FnMut(&mut Self) -> Option<Result<R, E>>) -> Vec<Result<R, E>> {
		let mut rs = Vec::new();

		while let Some(r) = self.try_take(&mut f) {
			rs.push(r);
		}

		rs
	}

	/// Attempts to match with the given function at least once, returning a list of all return values on success
	pub fn take_some<R, E>(&mut self, f: impl FnMut(&mut Self) -> Option<Result<R, E>>) -> Option<Vec<Result<R, E>>> {
		let ret = self.take_any(f);

		if ret.len() == 0 {
			None
		} else {
			Some(ret)
		}
	}

	/// Calls try_take on each function provided, returning the first Ok or Err value, or None
	pub fn take_choice<R, E>(&mut self, fs: Vec<Box<dyn FnMut(&mut Self) -> Option<Result<R, E>>>>) -> Option<Result<R, E>> {
		for mut f in fs {
			match self.try_take(|scanner| {
				f(scanner)
			}) {
				Some(ret) => return Some(ret),
				None => ()
			}
		}

		None
	}
}

#[cfg(test)]
mod test { // TODO: Unit tests
    use crate::utils::IntoResult;

    use super::Scanner;

	#[test]
	fn test_take_some() {
		let test = "hhhhhelo";

		let mut scanner = Scanner::new(test);

		assert_eq!(scanner.take_some(|scanner| {
			Some(scanner.take('h').into_result((), ()))
		}), Some(vec![Ok(()), Ok(()), Ok(()), Ok(()), Ok(())]));

		assert_eq!(scanner.cursor(), 5);

		assert_eq!(scanner.take_some(|scanner| {
			Some(scanner.take('e').into_result((), ()))
		}), Some(vec![Ok(())]));

		assert_eq!(scanner.cursor(), 6);
	}
}