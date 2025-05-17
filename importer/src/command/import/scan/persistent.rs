use itertools::{EitherOrBoth, Itertools};

use crate::global::{PERSISTENT_BEGIN, PERSISTENT_END};

#[derive(Debug)]
pub enum FetchPersistentError {
	NestedSections(usize),
	MismatchedDelimiters
}

pub fn get_persistent_sections<'a>(data: &'a str) -> Result<Vec<&'a str>, FetchPersistentError> {
	let starts = data.match_indices(PERSISTENT_BEGIN).map(|(index, _)| index);
	let ends = data.match_indices(PERSISTENT_END).map(|(index, _)| index);

	let mut out: Vec<&'a str> = Vec::new();

	let mut last_end = 0;

	for pair in starts.zip_longest(ends) {
		let EitherOrBoth::Both(start, end) = pair else { return Err(FetchPersistentError::MismatchedDelimiters); };

		if start < last_end { return Err(FetchPersistentError::NestedSections(start)); }
		
		last_end = end + PERSISTENT_END.len();
		out.push(&data[(start + PERSISTENT_BEGIN.len())..end]);
	}

	Ok(out)
}