use std::{fmt::{self, Display}, fs, path::PathBuf};

use console::style;
use dialoguer::theme::Theme;
use crate::{api::select::index::{self, Library}, core::{CollectionFilePathError, LibraryCache, LibraryIndexFormatError}};
use crate::api::select::selection::{Selection, SelectionOutput};

use crate::{global::API_VERSION, ProgramConfig, ProgramError};

#[derive(clap::Args, Debug)]
pub struct SelectArgs {
	#[arg(short, long)]
	file: PathBuf,
	#[arg(short, long)]
	out: PathBuf
}

pub struct ThemeFix;

impl Theme for ThemeFix {
	// * Copied from dialoguer exactly, except without colon in write.
	/// Formats an input prompt after selection.
	#[inline]
	fn format_input_prompt_selection(
		&self,
		f: &mut dyn fmt::Write,
		prompt: &str,
		sel: &str,
	) -> fmt::Result {
		write!(f, "{} {}", prompt, sel)
	}
}

pub enum SelectError {
	UserExit,
	AmbiguousCollections(Box<[String]>),
	DuplicateCollectionIds,
	DangerousCollectionName
}

impl From<SelectError> for ProgramError {
	fn from(value: SelectError) -> Self {
		match value {
			SelectError::UserExit => ProgramError::UserExit,
			SelectError::AmbiguousCollections(items) => todo!(),
			SelectError::DuplicateCollectionIds => todo!(),
			SelectError::DangerousCollectionName => todo!()
		}
	}
}

impl From<LibraryIndexFormatError> for SelectError {
	fn from(value: LibraryIndexFormatError) -> Self {
		match value {
			LibraryIndexFormatError::DuplicateIds => SelectError::DuplicateCollectionIds,
		}
	}
}

impl From<CollectionFilePathError> for SelectError {
	fn from(value: CollectionFilePathError) -> Self {
		match value {
			CollectionFilePathError::DangerousSegmentName => SelectError::DangerousCollectionName,
		}
	}
}

impl Display for SelectError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SelectError::UserExit => Ok(()),
			SelectError::AmbiguousCollections(items) => {
				write!(
					f,
					"{}: {}\n{}:\n{}\n{}: {}",
					style("Error").bold().red(),
					style("The selected document was found in multiple collections").bold(),
					"The following collections contain the selection",
					items.iter().map(|s| { format!(" - {}\n", style(s).cyan()) }).collect::<String>(),
					style("Help").cyan(), "Remove the document from all but at most one collection, then retry"
				)
			},
			SelectError::DuplicateCollectionIds => todo!(),
			SelectError::DangerousCollectionName => todo!()
		}
	}
}

pub fn select(config: &ProgramConfig, verbose: bool, args: SelectArgs) -> Result<(), SelectError> {
	let index_data: String = fs::read_to_string(args.file).unwrap();

	let index_file: index::Index = serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&index_data)).unwrap();
	if index_file.version != API_VERSION { eprint!("Unsupported index version"); todo!() }

	let index: index::User = serde_path_to_error::deserialize(index_file.index).unwrap();

	// * Select a library automatically if only one exists, or prompt the user to choose otherwise.
	let library = match TryInto::<&[Library; 1]>::try_into(index.libraries.as_ref()) {
		Ok(arr) => &arr[0],
		Err(_) => match dialoguer::FuzzySelect::with_theme(&ThemeFix {})
			.with_prompt("Select a library:")
			.items(&index.libraries.iter().map(|lib| &lib.name).collect::<Vec<_>>())
			.interact_opt()
			.unwrap()
		{
			Some(i) => &index.libraries[i],
			None => return Err(SelectError::UserExit),
		},
	};

	let cache: LibraryCache = LibraryCache::new(library)?;

	let document = match dialoguer::FuzzySelect::with_theme(&ThemeFix {})
		.with_prompt("Select a document:")
		.items(&library.documents.iter().map(|doc| &doc.title).collect::<Vec<_>>())
		.interact_opt()
		.unwrap() {
			Some(i) => &library.documents[i],
			None => return Err(SelectError::UserExit),
		};

	// println!("Importing as nested");
	// output_path.push(TryInto::<PathBuf>::try_into(cache.get_collection(collection_id).unwrap().get_path())?);
	// output_path.set_file_name(args.out);

	if !document.collection_ids.is_empty() {
		if document.collection_ids.len() > 1 {
			return Err(SelectError::AmbiguousCollections(
				document.collection_ids.iter().map(|id| {
					cache.get_collection(*id).unwrap().get_path().to_string()
				}).collect()
			));
		};
	}

	if verbose { println!("Writing output file to {}", args.out.to_string_lossy()); }

	fs::write(&args.out, serde_json::to_string(&SelectionOutput {
		version: API_VERSION,
		selection: Selection {
			library_id: library.id,
			document_id: document.id
		}
	}).unwrap()).unwrap();

	println!("\n{}: Selection complete", style("Info").bold());

	Ok(())
}