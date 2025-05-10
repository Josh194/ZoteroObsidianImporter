use std::{env, io::{self, BufWriter, Write}, path::PathBuf, process::{Command, ExitStatus}};

use clap::Parser as _;
use console::Term;
use interprocess::local_socket::{traits::Stream as _, GenericNamespaced, Stream, ToNsName};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorReason {
	UnsupportedPipeName,
	NavigateFailure,
	IPCInitFailure,
	ChildInitFailure,
	StreamFailure
}

#[derive(clap::Parser, Debug, Clone)]
#[command(version, propagate_version = true)]
struct Cli {
	#[arg(long)]
	pub working_directory: Option<PathBuf>,

	#[arg(long)]
	pub server: String,

	#[arg(long)]
	pub id: u32,

	#[arg(short, long)]
	pub wait: bool,

    #[clap(short, long, allow_hyphen_values = true, num_args = 1..)]
    command: Vec<String>
}

fn main() -> Result<(), ErrorReason> {
	let cli: Cli = Cli::parse();

	if let Some(path) = &cli.working_directory {
		env::set_current_dir(path).map_err(|_| ErrorReason::NavigateFailure)?;
	}

	let term: Term = Term::buffered_stdout();

	let mut client: BufWriter<Stream> = BufWriter::new(Stream::connect(
		cli.server.to_ns_name::<GenericNamespaced>().map_err(|_| ErrorReason::UnsupportedPipeName)?
	).map_err(|_| ErrorReason::IPCInitFailure)?);

	let status: ExitStatus = Command::new(&cli.command[0])
		.args(&cli.command[1..])
		.status()
		.map_err(|_| ErrorReason::ChildInitFailure)?;
	
	client.write_all(&shim_api::Msg::new(cli.id, &status).serialize()).map_err(|_| ErrorReason::StreamFailure)?;
	
	if cli.wait {
		print!("Press any key to continue . . . ");
		let _ = io::stdout().flush();

		let _ = term.read_key();
	}

	Ok(())
}