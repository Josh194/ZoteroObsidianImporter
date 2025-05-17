pub trait BorrowableIterator: Iterator {
	fn borrowed(&self) -> impl Iterator<Item = &Self::Item>;
}

impl<T> BorrowableIterator for std::vec::IntoIter<T> {
	fn borrowed(&self) -> impl Iterator<Item = &Self::Item> {
		self.as_slice().iter()
	}
}