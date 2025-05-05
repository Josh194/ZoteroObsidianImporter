use std::{env, fs::File, panic, sync::{OnceLock, RwLock}};

use console::style;

use crate::{config::LOG_NAME, panic::{panic_hook, PanicWrapper}, Cli, ProgramConfig};

// TODO: Use a safer lock when possible.
pub static LOG_FILE: RwLock<OnceLock<File>> = RwLock::new(OnceLock::new());

pub struct InitStatus {

}

pub struct PreInitStatus {
	log: bool
}

pub fn register_hook(args: &Cli, colour_logs: bool) {
	let verbose: bool = args.verbose;
	panic::set_hook(Box::new(move |info| panic_hook(PanicWrapper::new(info, colour_logs, verbose))));
}

pub fn preinit() -> Result<(), PreInitStatus> {
	let mut success: bool = true;

	let log = env::current_exe().map(|path| {
		File::create(path.with_file_name(LOG_NAME)).map(|file| {
			let target = LOG_FILE.write().unwrap_or_else(|e| {
				e.into_inner() // ! Is this dangerous?
			});
	
			let _ = target.set(file);
		}).map_err(|e| {
			println!("{}: {}", style("Warning").bold().yellow(), style("Unable to create log file").bold());
			println!("{}: {e}", style("Reason").bold());
			println!("Error messages will be printed to the console regardless of verbosity options");
		}).is_ok()
	}).map_err(|e| {		
		println!("{}: {}", style("Error").bold().red(), style("Failed to get executable path").bold());
		println!("{}: {e}", style("Reason").bold());
	}).is_ok();

	success &= log;

	if !success {
		return Err(PreInitStatus { log });
	}

	Ok(())
}

pub fn init(config: &ProgramConfig, args: &Cli) -> Result<(), InitStatus> {
	let mut success: bool = true;

	register_hook(args, config.log_coloring);

	if !success {
		return Err(InitStatus {});
	}

	Ok(())
}