use std::{env, fs::{self, File}, io::{self, Read, Seek}, path::{Path, PathBuf}};

use console::style;

use crate::{api::import::{self, annotation::Annotation}, util::versioned};
use crate::{global::{ANNOTATIONS_PREFIX, API_VERSION}, ProgramConfig, ProgramError};
use super::format::{annotation::{write_annotation, AnnnotationPersist, AnnotationExportError, AnnotationImportData, AnnotationTarget}, source::{write_source, SourceExportError, SourceImportData, SourcePersist, SourceTarget}, target::NTarget};
use super::scan::{notes::{get_note_files, NoteFetchError}, persistent::{get_persistent_sections, FetchPersistentError}};

#[derive(clap::Args, Debug)]
pub struct ImportArgs {
	#[arg(short, long)]
	file: PathBuf,
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

#[derive(Debug)]
struct NoteTarget {
	pub file: File,
	pub exists: bool,
	pub persists: Vec<String>
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

pub fn import(config: &ProgramConfig, verbose: bool, args: ImportArgs) -> Result<(), ProgramError> {
	let ProgramConfig { workspace_path, .. } = config;

	let export_file: String = fs::read_to_string(args.file).unwrap();

	let export: import::Export = versioned::deserialize_json_str_track(API_VERSION, &export_file).map_err(|e| {
		match e {
			versioned::Error::InvalidVersion(version) => {
				eprintln!("{}: {}", style("Error").bold().red(), style("Unsupported API version").bold());
				eprintln!("{}: An import query was made using version '{version}', but only '{API_VERSION}' is supported", style("Info").bold());

				ProgramError::UnsupportedAPIVersion
			},
			versioned::Error::Inner(e) => {
				eprintln!("{}: {}", style("Error").bold().red(), style("Invalid API query").bold());
				eprintln!("{}: {e}", style("Info").bold());

				ProgramError::InvalidAPIQuery
			},
		}
	})?;

	// * Load export file.
	// TODO: Could be multiple attachments.
	let import::Export { source, annotations } = export;

	// TODO: Need to improve this.
	let workspace_path: String = workspace_path.join(&source.title).to_str().unwrap().to_owned();

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
		fs::File::options().read(true).write(true).open(path).map_err(|error| {
			println!("Failed to open file!");
			println!("Filesystem IO error: {error}"); ProgramError::FilesystemError
		})
	}

	fn create_file<P: AsRef<Path>>(path: P) -> Result<fs::File, ProgramError> {
		fs::File::create_new(path).map_err(|error| {
			println!("Failed to create file!");
			println!("Filesystem IO error: {error}"); ProgramError::FilesystemError
		})
	}

	fn load_note(path: &PathBuf, exists: bool) -> Result<NoteTarget, ProgramError> {
		Ok(NoteTarget { file: if exists { open_file(path)? } else { create_file(path)? }, exists, persists: Vec::new() })
	}

	if verbose { println!("[DEBUG] - Current directory: {}\n", env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or("<UNKNOWN>".to_owned())); }

	println!("{}: Beginning import", style("Info").bold());

	fn log_note_output<P: AsRef<Path>>(path: P, exists: bool) {
		let file_name = path.as_ref().file_name().unwrap();

		if exists {
			println!("{} - {}", style("U").bold().cyan(), file_name.to_string_lossy());
		} else {
			println!("{} - {}", style("C").bold().green(), file_name.to_string_lossy());
		}
	}

	println!("{}:", style("Source").underlined());
	log_note_output(&files.source.path, files.source.exists);
	let mut source_target: NoteTarget = load_note(&files.source.path, files.source.exists)?;

	println!("{}:", style("Annotations").underlined());
	let annotation_targets: Vec<(Annotation, NoteTarget)> = files.annotations.map(|(z, file)| -> Result<_, ProgramError> {
		let path = PathBuf::from(&workspace_path).join(ANNOTATIONS_PREFIX).join(file.path).with_extension("md");

		log_note_output(&path, file.exists);
		Ok((z, load_note(&path, file.exists)?))
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

	println!("\n{}: Import complete", style("Finished").bold().green());

	Ok(())
}