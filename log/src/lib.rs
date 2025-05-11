use std::{fmt::Arguments, fs::File, io::{self, Write}, path::Path, sync::{OnceLock, RwLock}};

use console::style;

pub mod prelude;

// TODO: Use `format_args_nl` if it is ever stabilized.

struct Config {
	file: File,
	verbose: bool
}

// TODO: Use a safer lock when possible.
static CONFIG: RwLock<OnceLock<Config>> = RwLock::new(OnceLock::new());

pub fn set_config<P: AsRef<Path>>(file: P, verbose: bool) -> bool {
	File::create(file).map(|file| {
		let target = CONFIG.write().unwrap_or_else(|e| {
			e.into_inner() // ! Is this dangerous?
		});

		let _ = target.set(Config { file, verbose });
	}).map_err(|e| {
		println!("{}: {}", style("Warning").bold().yellow(), style("Unable to create log file").bold());
		println!("{}: {e}", style("Reason").bold());
		println!("Error messages will be printed to the console regardless of verbosity options");
	}).is_ok()
}

#[macro_export]
macro_rules! log {
	($($arg:tt)*) => {
		$crate::log(std::format_args!($($arg)*));
	};
}

#[macro_export]
macro_rules! logln {
	($($arg:tt)*) => {
		$crate::log(std::format_args!("{}\n", std::format_args!($($arg)*)));
	};
}

#[macro_export]
macro_rules! elog {
	($($arg:tt)*) => {
		$crate::elog(std::format_args!($($arg)*));
	};
}

#[macro_export]
macro_rules! elogln {
	($($arg:tt)*) => {
		$crate::elog(std::format_args!("{}\n", std::format_args!($($arg)*)));
	};
}

fn print(args: Arguments) {
	let _ = &mut io::stdout().write_fmt(args);
}

fn eprint(args: Arguments) {
	let _ = &mut io::stderr().write_fmt(args);
}

pub fn log(args: Arguments) {
	let mut log_guard = CONFIG.write().unwrap_or_else(|e| {
		e.into_inner() // ! Is this dangerous?
	});

	if let Some(Config { file, verbose }) = log_guard.get_mut() {
		if *verbose {
			print(args);
		}

		if let Err(e) = file.write_fmt(args) {
			eprintln!("{}: Unable to write error log", style("Warning").bold().yellow());
			eprintln!("{}: {e}", style("Reason").bold());
		}
	} else {
		print(args);
	}
}

pub fn elog(args: Arguments) {
	let mut log_guard = CONFIG.write().unwrap_or_else(|e| {
		e.into_inner() // ! Is this dangerous?
	});

	if let Some(Config { file, verbose }) = log_guard.get_mut() {
		if *verbose {
			eprint(args);
		}

		if let Err(e) = file.write_fmt(args) {
			eprintln!("{}: Unable to write error log", style("Warning").bold().yellow());
			eprintln!("{}: {e}", style("Reason").bold());
		}
	} else {
		eprint(args);
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn run() {
		log!("{}", 5);
		logln!("{}", 5);
		elog!("{}", 5);
		elogln!("{}", 5);
	}
}