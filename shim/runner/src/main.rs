use std::{env, fmt::Display, io::{BufReader, Read}, path::PathBuf, process::{self, Child, Command, ExitStatus, Stdio}};

use clap::Parser as _;
use interprocess::local_socket::{traits::Listener as _, GenericNamespaced, Listener, ListenerOptions, ToNsName};

// ! TODO: Replace named pipes with a less fragile alternative.

static SERVER_NAME: &str = "ZO_Importer_IPC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorReason {
	ChildError,
	NavigateFailure,
	UnsupportedPipeName,
	IPCInitFailure,
	ChildInitFailure,
	ChildWaitFailure,
	TerminalFailure,
	StreamFailure
}


#[derive(clap::Parser, Debug, Clone)]
#[command(version, propagate_version = true)]
struct Cli {
	#[arg(long)]
	pub working_directory: Option<PathBuf>,

	#[arg(long)] // Don't conflict with version.
	pub verbose: bool,

	#[clap(short, long, allow_hyphen_values = true, num_args = 1..)]
    command: Vec<String>
}

fn report_error<E: Display>(cli: &Cli, error: &E) {
	if cli.verbose { eprintln!("Error: {error}"); }
}

fn main() -> Result<(), ErrorReason> {
	let cli: Cli = Cli::parse();

	if let Some(path) = &cli.working_directory {
		env::set_current_dir(path).map_err(|e| -> ErrorReason { report_error(&cli, &e); ErrorReason::NavigateFailure })?;
	}

	let server: Listener = ListenerOptions::new()
		.name(SERVER_NAME.to_ns_name::<GenericNamespaced>()
			.map_err(|e| -> ErrorReason { report_error(&cli, &e); ErrorReason::UnsupportedPipeName })?)
		.create_sync()
		.map_err(|e| -> ErrorReason { report_error(&cli, &e); ErrorReason::IPCInitFailure })?;

	let id: u32 = process::id();

	let mut child: Child = {
		let mut command = Command::new("alacritty");

		command.args(["--command", "./proxy", "--wait", "--id", &id.to_string(), "--server", SERVER_NAME]);

		if let Some(path) = &cli.working_directory {
			command.arg("--working-directory").arg(path.as_os_str());
		}

		command
			.arg("--command").args(&cli.command)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null());

		command.spawn().map_err(|e| -> ErrorReason { report_error(&cli, &e); ErrorReason::ChildInitFailure })?
	};

	// TODO: Consider erroring on unexpected client failures to avoid the risk of deadlocking due to bugs.
	let (_code, success) = loop {
		match server.accept() {
			Ok(stream) => {
				let mut reader: BufReader<_> = BufReader::new(stream);

				let mut buffer: Vec<u8> = Vec::new();
				// TODO: Protect again client crashes potentially deadlocking us here.
				reader.read_to_end(&mut buffer).map_err(|e| -> ErrorReason { report_error(&cli, &e); ErrorReason::StreamFailure })?;

				match shim_api::Msg::deserialize(&buffer) {
					Ok(msg) => {
						if msg.id != id { if cli.verbose { eprintln!("Warning: Received a message from an unknown client\n    ({} != {})", msg.id, id); } continue;}

						break (msg.code, msg.success);
					},
					Err(e) => eprintln!("Warning: Received an invalid client message\n    {e:?}"),
				}
			},
			Err(e) => eprintln!("Warning: Failed to open a received client connection\n    {e}"),
		}
	};

	let status: ExitStatus = child.wait().map_err(|e| -> ErrorReason { report_error(&cli, &e); ErrorReason::ChildWaitFailure })?;

	if !status.success() { if cli.verbose { eprintln!("Error: Terminal failed"); } return Err(ErrorReason::TerminalFailure); }

	if success { Ok(()) } else { Err(ErrorReason::ChildError) }
}