/// Checks if a string corresponds to a single valid path segment.
/// 
/// Is extremely restrictive to be safe, only allowing basic ASCII alphanumeric characters.
pub fn is_path_segment<T: AsRef<str>>(segment: T) -> bool {
	for character in segment.as_ref().chars() {
		if !character.is_ascii_alphanumeric() { return false; }
	}

	true
}