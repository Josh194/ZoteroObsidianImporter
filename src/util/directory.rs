use std::{fs, io, path::{Path, PathBuf}};

/// Returns a list of relative paths to all files in a directory matching an extension.
pub fn get_files_ext<P: AsRef<Path>>(path: P, extension: &str) -> Result<Vec<PathBuf>, io::Error> {
	let mut out: Vec<PathBuf> = Vec::new();

	for entry in fs::read_dir(path)? {
		let entry = entry?;

		let path = entry.path();

		if entry.file_type()?.is_file() && path.extension().map(|ext| { ext == extension }).unwrap_or(false) {
			out.push(path);
		}
	}

	Ok(out)
}