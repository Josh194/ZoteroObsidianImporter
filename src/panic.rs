use std::{backtrace::{Backtrace, BacktraceStatus}, fmt::Display, io::Write, panic::PanicHookInfo, thread};

use console::style;

use crate::global;

pub fn panic_hook(panic: PanicWrapper) {
	let mut log_guard = global::LOG_FILE.write().unwrap_or_else(|e| {
		e.into_inner() // ! Is this dangerous?
	});

	let panic_result: String = format!("{panic}");

	println!();

	if let Some(file) = log_guard.get_mut() {
		if panic.verbose {
			println!("{panic_result}");
		}
		
		if let Err(_e) = file.write_all(panic_result.as_bytes()) {
			println!("{}: Unable to write error log", style("Warning").bold().yellow());
		}
	} else {
		println!("{panic_result}");
	}

	println!("{}: {}\nPlease consider filing a bug report along with the crash logs.", style("Error").bold().red(), style("Fatal internal error encountered").bold());
}

#[derive(Debug, Clone, Copy)]
pub struct PanicWrapper<'a, 'b> {
	info: &'a PanicHookInfo<'b>,
	colour_logs: bool,
	verbose: bool
}

impl<'a, 'b> PanicWrapper<'a, 'b> {
	pub fn new(info: &'a PanicHookInfo<'b>, colour_logs: bool, verbose: bool) -> Self {
		Self { info, colour_logs, verbose }
	}
}

impl<'a, 'b> Display for PanicWrapper<'a, 'b> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let thread = thread::current();

		let Self { info, colour_logs: _, verbose: _ } = *self;

		writeln!(f, "{}: {}", style("Error").bold().red(), style(format!("Thread '{}' panicked", thread.name().map(|s| s.to_owned()).unwrap_or_else(|| format!("{:?}", thread.id())))).bold())?;

		if let Some(loc) = info.location() {
			writeln!(f, "{}: Location was {}", style("Info").bold(), style(format!("{}#{}:{}", loc.file(), loc.line(), loc.column())).cyan())?;
		} else {
			writeln!(f, "{}: Unable to get location details", style("Warning").bold().yellow())?;
		}

		let payload = info.payload();

		write!(f, "{}: ", style("Reason").bold())?;

		if let Some(s) = payload.downcast_ref::<String>() { writeln!(f, "{s}")?;
		} else if let Some(s) = payload.downcast_ref::<&str>() { writeln!(f, "{s}")?;
		} else { writeln!(f, "{payload:#?}")?; }

		let trace = Backtrace::capture();

		match trace.status() {
			BacktraceStatus::Unsupported => writeln!(f, "{}: Backtrace not captured as it is unsupported in this environment", style("Info").bold())?,
			BacktraceStatus::Disabled => writeln!(f, "{}: Backtrace not captured as it is currently disabled; set the environment variable `RUST_BACKTRACE` to `1` in order to display one", style("Info").bold())?,
			BacktraceStatus::Captured => writeln!(f, "{}", trace)?,
			status => writeln!(f, "{}: Received an unknown status while trying to capture a backtrace ({status:?})", style("Info").bold())?,
		}

		Ok(())
	}
}