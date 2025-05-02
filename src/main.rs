use std::{fs::{self}, process::{ExitCode, Termination}};

use clap::Parser as _;
use console::{style, user_attended_stderr};
use serde::Deserialize;

mod document;
mod text;
mod import;
mod config;
mod util;
mod scan;
mod format;
mod command;

// ! TODO: Sanitize data everywhere.

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
	fn from(_: dialoguer::Error) -> Self {
		ProgramError::InteractError
	}
}

#[derive(Debug, Clone, Deserialize)]
struct ProgramConfig {
	import_path: String,
	workspace_path: String
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

	// ! file.sync_data()

	command::import::import(&config, debug)
}