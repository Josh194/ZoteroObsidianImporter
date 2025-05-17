use std::fmt::Display;

/// * Currently unusable since this does not account for the formatting escape codes
struct SurroundedString<T: Display> {
	value: T
}

impl<T: Display> SurroundedString<T> {
	pub fn new(value: T) -> Self {
		Self { value }
	}
}

fn surround<T: Display>(value: T) -> SurroundedString<T> {
	SurroundedString::new(value)
}

impl<T: Display> Display for SurroundedString<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let content = self.value.to_string();
		let surround = "=".repeat(content.len());

		write!(f, "{}\n{}\n{}", surround, content, surround)
	}
}