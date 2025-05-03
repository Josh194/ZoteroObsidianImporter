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

// ! TODO: Sanitize data everywhere.

#[derive(Debug, Clone, Copy)]
enum ProgramError {
	UserExit,
	Unattended,
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
				ExitCode::SUCCESS
			},
			Err(error) => {
				match error {
					ProgramError::UserExit => (),
					_ => println!("{}: {error:?}", style("Error").bold().red())
				}

				println!("\n{}: Import failed", style("Error").bold().red());

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

	let cli = Cli::parse();

	if !user_attended_stderr() { return Err(ProgramError::Unattended); } // TODO: Just auto-fail prompts if unattended

	let config_str: String = fs::read_to_string("config.json").unwrap();
	
	let config_file: ConfigFile = serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&config_str)).unwrap();
	if config_file.version != CONFIG_VERSION { eprint!("Unsupported config version"); todo!() }

	let config: ProgramConfig = serde_path_to_error::deserialize(config_file.config).unwrap();

	if let Err(e) = fs::create_dir_all(&config.data_path) {
		panic!("Cannot access data directory!");
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