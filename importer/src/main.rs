#![deny(unsafe_op_in_unsafe_fn)]

use std::{env, fs::{self}, path::PathBuf, process::{ExitCode, Termination}};

use clap::Parser as _;
use command::{import::ImportArgs, select::SelectArgs};
use global::CONFIG_VERSION;
use console::style;
use global::init;
use serde::Deserialize;
use util::versioned;

/// Global state, initialization, and context.
mod global;
/// Panic handling and logging.
mod panic;
/// Logging framework.
mod log;
/// General utilities.
mod util;
/// Core structures and operations for working with queries.
mod core;
/// API structures and serialization-related routines.
mod api;
/// Command-specific routines.
mod command;
/// Cross-query structures and operations.
mod db;

// ! TODO: Sanitize data everywhere.

#[derive(Debug, Clone, Copy)]
enum ProgramError {
	UserExit,
	Unattended,
	InaccessibleConfig,
	InvalidConfig,
	InvalidAPIQuery,
	UnsupportedConfigVersion,
	UnsupportedAPIVersion,
	BadImportFormat,
	BadIndexFormat,
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
#[serde(deny_unknown_fields)]
struct ConfigFile {
	version: i64,
	config: serde_json::Value
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgramConfig {
	#[serde(default)]
	log_coloring: bool,
	data_path: PathBuf,
	workspace_path: PathBuf
}

struct ProgramResult {
	result: Result<(), ProgramError>
}

impl Termination for ProgramResult {
	fn report(self) -> ExitCode {
		match self.result {
			Ok(_) => ExitCode::SUCCESS,
			Err(error) => {
				eprint!("\n{}: Import failed", style("Error").bold().red());

				match error {
					ProgramError::UserExit => eprintln!(),
					_ => eprintln!(" ({error:?})")
				}

				ExitCode::FAILURE
			},
		}
	}
}

#[derive(clap::Parser, Debug)]
#[command(version, propagate_version = true)]
struct Cli {
	#[arg(long)] // Don't conflict with version.
	verbose: bool,

	#[command(subcommand)]
	command: Command
}

#[derive(clap::Subcommand, Debug)]
enum Command {
	Select(SelectArgs),
	Import(ImportArgs)
}

fn main() -> ProgramResult {
	ProgramResult { result: run() }
}

fn run() -> Result<(), ProgramError> {
	unsafe { env::set_var("RUST_BACKTRACE", "1") };

	// ! TODO: This is checking the wrong thing; need to look at stdin, not any of the outputs.
	if !console::user_attended_stderr() { return Err(ProgramError::Unattended); } // TODO: Just auto-fail prompts if unattended

	// * Parse cli arguments.
	let cli = Cli::parse();

	// * Perform preinitialization and set the panic hook with default values.

	if let Err(e) = init::preinit() {

	}

	init::register_hook(&cli, false);
	
	// * Load the config file.

	let config_file: String = fs::read_to_string("config.json").map_err(|e| {
		eprintln!("{}: {}", style("Error").bold().red(), style("Failed to read config file").bold());
		eprintln!("{}: {e}", style("Reason").bold());

		ProgramError::InaccessibleConfig
	})?;

	let config: ProgramConfig = versioned::deserialize_json_str_track(CONFIG_VERSION, &config_file).map_err(|e| {
		match e {
			versioned::Error::InvalidVersion(version) => {
				eprintln!("{}: {}", style("Error").bold().red(), style("Unsupported config version").bold());
				eprintln!("{}: The provided config uses version '{version}', but only '{CONFIG_VERSION}' is supported", style("Info").bold());

				ProgramError::UnsupportedConfigVersion
			},
			versioned::Error::Inner(e) => {
				eprintln!("{}: {}", style("Error").bold().red(), style("Invalid config file format").bold());
				eprintln!("{}: {e}", style("Info").bold());

				ProgramError::InvalidConfig
			},
		}
	})?;

	if let Err(e) = fs::create_dir_all(&config.data_path) {
		eprintln!("{}: {}", style("Error").bold().red(), style("Unable to access data directory").bold());
		eprintln!("{}: {e}", style("Reason").bold());

		return Err(ProgramError::FilesystemError);
	}

	// * Complete initialization with the now available config.
	if let Err(e) = init::postinit(&config, &cli) {
		
	}
	
	let Cli { verbose, command } = cli;

	// * Execute the selected command.

	match command {
		Command::Select(select_args) => {
			if let Err(e) = command::select::select(&config, verbose, select_args) {
				eprintln!("\n{e}"); return Err(e.into());
			}
		},
		Command::Import(import_args) => {
			command::import::import(&config, verbose, import_args)?;
		},
	}

	// ! file.sync_data()

	Ok(())
}