use std::fs::File;

#[derive(Debug)]
pub struct NoteTarget<'a, Data, Persist> {
	/// The file to write the data to.
	pub file: &'a mut File,
	/// All relevent import data.
	pub data: Data,
	/// Persisted data from the previous contents, if a matching file was located.
	pub persist: Option<Persist>
}

pub trait NTarget {
	type Data;
	type Persist;
	type Error;

	fn new(file: &mut File, data: Self::Data, persist: Option<Self::Persist>) -> Self;
	fn write(&mut self) -> Result<(), Self::Error>;
}