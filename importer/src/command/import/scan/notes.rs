use std::{collections::HashMap, fs, io, path::{Path, PathBuf}, vec};

use console::style;
use itertools::{Itertools, ZipEq};

use crate::{global::{ANNOTATIONS_PREFIX, SOURCE_PREFIX}, util::{directory::get_files_ext, iter::BorrowableIterator}};

#[derive(Debug, Clone)]
pub struct AnnotationFile {
	pub path: PathBuf,
	pub exists: bool
}

#[derive(Debug, Clone)]
pub struct SourceFiles<AIter: Iterator, FIter: Iterator<Item = AnnotationFile>> {
	pub source: AnnotationFile,
	pub annotations: ZipEq<AIter, FIter>,
	pub residuals: Vec<PathBuf>
}

#[derive(Debug)]
pub enum NoteFetchError {
	UnrecognizedSources,
	Interact(dialoguer::Error),
	Filesystem(io::Error)
}

impl From<io::Error> for NoteFetchError {
	fn from(value: io::Error) -> Self {
		Self::Filesystem(value)
	}
}

impl From<dialoguer::Error> for NoteFetchError {
	fn from(value: dialoguer::Error) -> Self {
		Self::Interact(value)
	}
}

type AnnotOutIter = vec::IntoIter<AnnotationFile>;

pub fn get_note_files<
	'a,
	P: AsRef<Path>,
	AIter: BorrowableIterator,
	F: FnMut(&AIter::Item) -> AName,
	AName: AsRef<str>
> (path: P, source_name: &str, annotations: AIter, a_map: F) -> Result<SourceFiles<AIter, AnnotOutIter>, NoteFetchError> {
	let source_path = path.as_ref().join(SOURCE_PREFIX);
	let annotation_path = path.as_ref().join(ANNOTATIONS_PREFIX);

	fs::create_dir_all(&annotation_path)?;
	
	let mut residuals = get_files_ext(&source_path, "md")?;

	let source_exists: bool = residuals.iter().position(|elem| elem.file_stem().unwrap() == source_name).map(|index| {
		residuals.swap_remove(index)
	}).is_some();

	let mut out: Vec<AnnotationFile> = annotations.borrowed().map(a_map).map(|s| AnnotationFile { path: s.as_ref().into(), exists: false }).collect();
	let mut keys: HashMap<&Path, &mut bool> = out.iter_mut().map(|a_file| (a_file.path.as_path(), &mut a_file.exists)).collect();

	let mut db = get_files_ext(annotation_path, "md")?;

	// TODO: Use `Vec::extract_if` to avoid the `path.clone()` when it is stabilized (should be May 15 2025 for rustc 1.87).
	// TODO: Clean this up in general as well if possible. Probably try to get rip of the string conversion.
	db.retain(|path| {
		keys.get_mut(Path::new(path.file_stem().unwrap())).map(|result| **result = true ).is_none()
	});

	residuals.extend(db);

	if !residuals.is_empty() {
		if !query_delete_files(residuals.iter())? { return Err(NoteFetchError::UnrecognizedSources) }
	}

	Ok(SourceFiles {
		source: AnnotationFile { path: source_path.join(source_name).with_extension("md"), exists: source_exists },
		annotations: annotations.zip_eq(out.into_iter()),
		residuals
	})
}

fn query_delete_files<I: Iterator<Item: AsRef<Path>>>(files: I) -> Result<bool, dialoguer::Error> {
	dialoguer::Confirm::new()
		.with_prompt(format!(
			"{}: {}\n{}:\n{}\n{}\n",
			style("Warning").bold().yellow(),
			style("Output notes directory contains unrecognized notes").bold(),
			"Continuing will cause the following files to be permanently deleted, with any persistent notes being lost",
			files.map(|s| { format!(" - {}\n", style(s.as_ref().to_string_lossy()).cyan()) }).collect::<String>(),
			style("Do you still want to proceed?").bold().magenta(),
		)).default(false)
		.report(false) // dialoguer bug causes this to sometimes cause duplication.
		.interact()
}