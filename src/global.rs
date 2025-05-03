use std::{fs::File, panic, sync::{OnceLock, RwLock}};

use console::style;

use crate::{config::LOG_NAME, panic::{panic_hook, PanicWrapper}, Cli, ProgramConfig};

// TODO: Use a safer lock when possible.
pub static LOG_FILE: RwLock<OnceLock<File>> = RwLock::new(OnceLock::new());

pub struct InitStatus {
	log: bool
}

pub fn init(config: &ProgramConfig, args: &Cli) -> Result<(), InitStatus> {
	let mut success: bool = true;
	
	let log = File::create(config.data_path.with_file_name(LOG_NAME)).map(|file| {
		let target = LOG_FILE.write().unwrap_or_else(|e| {
			e.into_inner() // ! Is this dangerous?
		});

		let _ = target.set(file);
	}).inspect(|_| {
		success = false;

		println!("{}: {}\nError messages will be printed to the console regardless of verbosity options", style("Warning").bold().yellow(), style("Unable to create log file").bold());
	} ).is_ok();

	{
		let colour_logs = config.log_coloring;
		let verbose = args.verbose;

		panic::set_hook(Box::new(move |info| panic_hook(PanicWrapper::new(info, colour_logs, verbose))));
	}

	if !success {
		return Err(InitStatus { log });
	}

	Ok(())
}