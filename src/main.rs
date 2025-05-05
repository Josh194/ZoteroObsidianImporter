#![deny(unsafe_op_in_unsafe_fn)]

use std::{env, fs::{self}, path::PathBuf, process::{ExitCode, Termination}};

use clap::Parser as _;
use command::{import::ImportArgs, select::SelectArgs};
use config::CONFIG_VERSION;
use console::{style, user_attended_stderr};
use serde::Deserialize;

mod document;
// mod text;
mod import;
mod config;
mod util;
mod scan;
mod format;
mod command;
mod global;
mod log;
mod panic;
mod api;

// ! TODO: Sanitize data everywhere.

#[derive(Debug, Clone, Copy)]
enum ProgramError {
	UserExit,
	Unattended,
	InaccessibleConfig,
	InvalidConfig,
	UnsupportedConfig,
	UnsupportedAPI,
	BadImportFormat,
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
	import_path: PathBuf,
	workspace_path: PathBuf
}

struct ProgramResult {
	result: Result<(), ProgramError>
}

impl Termination for ProgramResult {
	fn report(self) -> ExitCode {
		match self.result {
			Ok(_) => {
				ExitCode::SUCCESS
			},
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

	if !user_attended_stderr() { return Err(ProgramError::Unattended); } // TODO: Just auto-fail prompts if unattended

	let cli = Cli::parse();

	if let Err(e) = global::preinit() {

	}

	global::register_hook(&cli, false);

	let config_str: String = fs::read_to_string("config.json").map_err(|e| {
		eprintln!("{}: {}", style("Error").bold().red(), style("Failed to read config file").bold());
		eprintln!("{}: {e}", style("Reason").bold());

		ProgramError::InaccessibleConfig
	})?;
	
	let process_config_error = |e: serde_path_to_error::Error<serde_json::Error>| {
		eprintln!("{}: {}", style("Error").bold().red(), style("Invalid config file format").bold());
		eprintln!("{}: {e}", style("Info").bold());

		ProgramError::InvalidConfig
	};

	let config_file: ConfigFile = serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&config_str)).map_err(process_config_error)?;

	if config_file.version != CONFIG_VERSION {
		eprintln!("{}: {}", style("Error").bold().red(), style("Unsupported config version").bold());
		eprintln!("{}: The provided config uses version '{}', but only '{}' is supported", style("Info").bold(), config_file.version, CONFIG_VERSION);

		return Err(ProgramError::UnsupportedConfig);
	}

	let config: ProgramConfig = serde_path_to_error::deserialize(config_file.config).map_err(process_config_error)?;

	if let Err(e) = fs::create_dir_all(&config.data_path) {
		eprintln!("{}: {}", style("Error").bold().red(), style("Unable to access data directory").bold());
		eprintln!("{}: {e}", style("Reason").bold());

		return Err(ProgramError::FilesystemError);
	}

	if let Err(e) = global::init(&config, &cli) {
		
	}
	
	let Cli { verbose, command } = cli;

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