use std::{env, fmt::Display, fs::{self, File}, io::{self, Read, Seek}, path::{Path, PathBuf}, process::{ExitCode, Termination}, str::FromStr, vec};

use clap::Parser as _;
use config::ANNOTATIONS_PREFIX;
use console::{style, user_attended_stderr};
use document::{annotation::ZAnnotation, doc::Document};
use format::{annotation::{write_annotation, AnnnotationPersist, AnnotationExportError, AnnotationImportData, AnnotationTarget}, source::{write_source, SourceExportError, SourceImportData, SourcePersist, SourceTarget}, NTarget};
use import::{annotations::import_annotations, source::{import_source, DocumentMeta, ImportSourceError}};
use itertools::Itertools;
use scan::{notes::{get_note_files, BorrowableIterator, NoteFetchError}, persistent::{get_persistent_sections, FetchPersistentError}};
use serde::Deserialize;

mod document;
mod text;
mod import;
mod config;
mod util;
mod scan;
mod format;

// ! TODO: Sanitize data everywhere.

#[derive(Debug)]
struct NoteTarget {
	pub file: File,
	pub exists: bool,
	pub persists: Vec<String>
}

#[derive(Debug)]
enum ParsePersistsError {
	Io(io::Error),
	ParsePersists(FetchPersistentError)
}

impl From<io::Error> for ParsePersistsError {
	fn from(value: io::Error) -> Self {
		ParsePersistsError::Io(value)
	}
}

impl From<FetchPersistentError> for ParsePersistsError {
	fn from(value: FetchPersistentError) -> Self {
		ParsePersistsError::ParsePersists(value)
	}
}

impl NoteTarget {
	pub fn write<N: NTarget<Persist: From<String>>>(&mut self, data: N::Data, persist: N::Persist) -> Result<(), N::Error> {
		let persist = if self.exists {
			self.parse_persists().unwrap(); self.file.set_len(0).unwrap(); self.file.rewind().unwrap(); Some(self.persists[0].as_str())
		} else { None };
		
		N::new(
			&mut self.file,
			data,
			persist.map(|p| N::Persist::from(p.to_owned())) // TODO: Accept a `&str` here.
		).write()
	}

	pub fn parse_persists(&mut self) -> Result<(), ParsePersistsError> {
		let mut data = String::new();
		self.file.read_to_string(&mut data)?;

		self.persists = get_persistent_sections(&data)?.into_iter().map(|s| s.to_owned()).collect();
		Ok(())
	}
}

#[derive(Debug, Clone, Copy)]
enum ProgramError {
	UserExit,
	Unattended,
	BadImportFormat,
	PDFLoadError,
	PDFParseError,
	AnnotationParseError,
	FilesystemError,
	InteractError,
	YAMLDeserializeError
}

impl From<dialoguer::Error> for ProgramError {
	fn from(value: dialoguer::Error) -> Self {
		ProgramError::InteractError
	}
}

#[derive(Debug, Clone, Deserialize)]
struct ProgramConfig {
	import_path: String,
	workspace_path: String
}

impl<T> BorrowableIterator for vec::IntoIter<T> {
	fn borrowed(&self) -> impl Iterator<Item = &Self::Item> {
		self.as_slice().iter()
	}
}

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

fn get_sur(n: usize) -> String {
	"=".repeat(n)
}

struct ProgramResult {
	result: Result<(), ProgramError>
}

impl Termination for ProgramResult {
	fn report(self) -> ExitCode {
		match self.result {
			Ok(_) => {
				let sur = get_sur(24);
				println!("{sur}\n{}: Import complete\n{sur}", style("Success").bold().green());

				ExitCode::SUCCESS
			},
			Err(error) => {
				match error {
					ProgramError::UserExit => (),
					_ => println!("{}: {error:?}", style("Error").bold().red())
				}

				let sur = get_sur(22);
				println!("{sur}\n{}: Import failed\n{sur}", style("Exiting").bold().red());

				ExitCode::FAILURE
			},
		}
	}
}

#[derive(clap::Parser, Debug)]
#[command(version)]
struct Args {
	#[arg(long)]
	debug: bool
}

fn main() -> ProgramResult {
	ProgramResult { result: run() }
}

fn run() -> Result<(), ProgramError> {
	let Args { debug } = Args::parse();

	if !user_attended_stderr() { return Err(ProgramError::Unattended); } // TODO: Just auto-fail prompts if unattended

	let config_str: String = fs::read_to_string("config.json").unwrap();
	let config: ProgramConfig = serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&config_str)).unwrap();

	let ProgramConfig { import_path, mut workspace_path } = config;

	// * Load exported source metadata.
	let source: DocumentMeta = match import_source(&import_path) {
		Ok(val) => val,
		Err(e) => {
			println!("Error loading import metadata!");

			match e {
				ImportSourceError::Filesystem(error) => { println!("Filesystem IO error: {error}"); return Err(ProgramError::FilesystemError); },
				ImportSourceError::InvalidJson(error) => { println!("Invalid format: {error}"); return Err(ProgramError::BadImportFormat); },
				ImportSourceError::WrongItemCount => { println!("Unsupported number of items"); return Err(ProgramError::BadImportFormat); },
				ImportSourceError::InvalidItemFormat => { println!("Invalid item format"); return Err(ProgramError::BadImportFormat); }
			}
		},
	};

	// TODO: Need to improve this.
	workspace_path = PathBuf::from_str(&workspace_path).unwrap().join(&source.citation_key).to_str().unwrap().to_owned();

	// * Load exported PDF.
	// TODO: Could be multiple attachments.
	let annotation_document: Document = match lopdf::Document::load(PathBuf::from(&import_path).join(&source.attachments[0].path)) {
		Ok(pdf) => {
			if debug {
				println!("{:?}", pdf.extract_text(&pdf.get_pages().keys().copied().collect_vec()).unwrap().replace("\n", ""));

				for page in pdf.page_iter() {
					//println!("{:?}", pdf.get_page_annotations(page));
				}
				
				return Ok(());
			}

			match pdf.try_into() {
				Ok(val) => val,
				Err(error) => { println!("Error parsing import PDF: {error:?}"); return Err(ProgramError::PDFParseError); }
			}
		},
		Err(error) => { println!("Error loading import PDF: {error}"); return Err(ProgramError::PDFLoadError); }
	};

	// * Parse annotations from PDF.
	let annotations = match import_annotations(&annotation_document) {
		Ok(val) => val,
		Err(error) => { println!("Error importing annotations: {error:?}"); return Err(ProgramError::AnnotationParseError); }
	};

	// * Determine current output directory contents, relative to the target output.
	let files = match get_note_files(&workspace_path, &source.file_name(), annotations.into_iter(), |a| { format!("{} {}", source.short_name(), a.key) }) {
		Ok(val) => val,
		Err(e) => {
			println!("Error determining existing note structure!");

			match e {
				NoteFetchError::UnrecognizedSources => { return Err(ProgramError::UserExit); },
				NoteFetchError::Interact(error) => { println!("Console interaction error: {error}"); return Err(error.into()) },
				NoteFetchError::Filesystem(error) => { println!("Filesystem IO error: {error}"); return Err(ProgramError::FilesystemError); }
			}
		}
	};

	// * Delete residual (unknown) notes. This is subject to user confirmation in `get_note_files``
	for residual in &files.residuals {
		if let Err(e) = fs::remove_file(residual) {
			println!("Failed to remove file: {}\nFilesystem IO error: {e}", residual.to_string_lossy());
		}
	}

	fn open_file<P: AsRef<Path>>(path: P) -> Result<fs::File, ProgramError> {
		println!("Updating - {:?}", path.as_ref());

		fs::File::options().read(true).write(true).open(path).map_err(|error| {
			println!("Failed to open file!");
			println!("Filesystem IO error: {error}"); ProgramError::FilesystemError
		})
	}

	fn create_file<P: AsRef<Path>>(path: P) -> Result<fs::File, ProgramError> {
		println!("Creating - {:?}", path.as_ref());

		fs::File::create_new(path).map_err(|error| {
			println!("Failed to create file!");
			println!("Filesystem IO error: {error}"); ProgramError::FilesystemError
		})
	}

	fn load_note(path: &PathBuf, exists: bool) -> Result<NoteTarget, ProgramError> {
		Ok(NoteTarget { file: if exists { open_file(path)? } else { create_file(path)? }, exists, persists: Vec::new() })
	}

	println!("[DEBUG] - Current directory: {:?}\n\nBeginning import:", env::current_dir().unwrap());

	let mut source_target: NoteTarget = load_note(&files.source.path, files.source.exists)?;
	let annotation_targets: Vec<(ZAnnotation, NoteTarget)> = files.annotations.map(|(z, file)| -> Result<_, ProgramError> {
		Ok((z, load_note(&PathBuf::from(&workspace_path).join(ANNOTATIONS_PREFIX).join(file.path).with_extension("md"), file.exists)?))
	}).collect::<Result<Vec<_>, _>>()?;

	// * Write output notes.

	let persist = if source_target.exists { source_target.parse_persists().unwrap(); source_target.file.set_len(0).unwrap(); source_target.file.rewind().unwrap(); Some(source_target.persists[0].as_str()) } else { None };
	if let Err(e) = write_source(SourceTarget {
		file: &mut source_target.file,
		data: SourceImportData { source: &source },
		persist: persist.map(|s| SourcePersist { content_section: s.to_owned() })
	}) {
		println!("Error exporting source note!");

		match e {
			SourceExportError::Io(error) => { println!("Filesystem IO error: {error}"); return Err(ProgramError::FilesystemError); },
			SourceExportError::PropertyDeserialize(error) => { println!("Note property formatting error: {error}"); return Err(ProgramError::YAMLDeserializeError); },
		}
	}
	
	for (annotation, mut target) in annotation_targets {
		let persist = if target.exists { target.parse_persists().unwrap(); target.file.set_len(0).unwrap(); target.file.rewind().unwrap(); Some(target.persists[0].as_str()) } else { None };
		
		if let Err(e) = write_annotation(AnnotationTarget {
			file: &mut target.file,
			data: AnnotationImportData { source: &source, annot: annotation },
			persist: persist.map(|s| AnnnotationPersist { content_section: s.to_owned() })
		}) {
			println!("Error exporting annotation note!");

			match e {
				AnnotationExportError::Io(error) => { println!("Filesystem IO error: {error}"); return Err(ProgramError::FilesystemError); },
				AnnotationExportError::PropertyDeserialize(error) => { println!("Note property formatting error: {error}"); return Err(ProgramError::YAMLDeserializeError); },
			}
		}
	}

	// ! file.sync_data()

	Ok(())
}