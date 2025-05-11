use std::{env, fs, io::{BufReader, Read}, path::{Path, PathBuf}, process::{self, Child, Command, ExitStatus, Stdio}};

use clap::Parser as _;
use interprocess::local_socket::{traits::Listener as _, GenericFilePath, GenericNamespaced, Listener, ListenerOptions, Name, ToFsName, ToNsName};
use log::prelude::*;

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

	// TODO: Implement functionality here.
	#[arg(long)]
	pub log: bool,

	#[arg(long)] // Don't conflict with version.
	pub verbose: bool,

	#[clap(short, long, allow_hyphen_values = true, num_args = 1..)]
    command: Vec<String>
}

// * Mac fails to find any of the executables if we do not give an absolute path, hence the the need for this.
// * Additionally Zotero does not seem to properly pass in the system environment variables on Mac, so this is needed even for executables on the path.
fn resolve_path<P: AsRef<Path>>(path: P) -> PathBuf {
	env::current_dir().unwrap_or_default().join(path)
}

fn find_terminal() -> PathBuf {
	if cfg!(windows) {
		Path::new("alacritty").to_owned()
	} else {
		resolve_path("alacritty")
	}
}

// * Required because interprocess currently fails to cleanup properly on some platforms (is the name not being dropped somehow?).
// TODO: This is concerningly fragile (no guarantee we are removing the correct path).
fn reclaim_ipc<P: AsRef<Path>>(path: P) {
	let _ = fs::remove_file(path);
}

fn main() -> Result<(), ErrorReason> {
	let cli: Cli = Cli::parse();

	if let Ok(path) = env::current_exe() {
		log::set_config(path.with_file_name("shim.log"), cli.verbose);
	}

	if let Some(path) = &cli.working_directory {
		env::set_current_dir(path).map_err(|e| -> ErrorReason { elogln!("Error: Failed to navigate to working directory\n    {e}"); ErrorReason::NavigateFailure })?;
	}

	let ipc_name: Name = if cfg!(unix) {
		let path = Path::new("/tmp").join(SERVER_NAME);
		reclaim_ipc(&path);

		path.to_fs_name::<GenericFilePath>()
	} else {
		SERVER_NAME.to_ns_name::<GenericNamespaced>()
	}.map_err(|e| -> ErrorReason { elogln!("Error: Pipe name is unsupported\n    {e}"); ErrorReason::UnsupportedPipeName })?;

	let server: Listener = ListenerOptions::new()
		.name(ipc_name)
		.create_sync()
		.map_err(|e| -> ErrorReason { elogln!("Error: Failed to initialize IPC\n    {e}"); ErrorReason::IPCInitFailure })?;

	let id: u32 = process::id();

	let mut child: Child = {
		let mut command = Command::new(find_terminal());

		command
			.arg("--command").arg(resolve_path("proxy"))
			.args(["--wait", "--id", &id.to_string(), "--server", SERVER_NAME]);

		if let Some(path) = &cli.working_directory {
			command.arg("--working-directory").arg(path.as_os_str());
		}

		command
			.arg("--command").args(&cli.command)
			.stdin(Stdio::null());

		command.spawn().map_err(|e| -> ErrorReason { elogln!("Error: Failed to initialize child\n    {e}"); ErrorReason::ChildInitFailure })?
	};

	// TODO: Consider erroring on unexpected client failures to avoid the risk of deadlocking due to bugs.
	let (_code, success) = loop {
		match server.accept() {
			Ok(stream) => {
				let mut reader: BufReader<_> = BufReader::new(stream);

				let mut buffer: Vec<u8> = Vec::new();
				// TODO: Protect again client crashes potentially deadlocking us here.
				reader.read_to_end(&mut buffer).map_err(|e| -> ErrorReason { elogln!("Error: Failed to read from steam\n    {e}"); ErrorReason::StreamFailure })?;

				match shim_api::Msg::deserialize(&buffer) {
					Ok(msg) => {
						if msg.id != id { if cli.verbose { elogln!("Warning: Received a message from an unknown client\n    ({} != {})", msg.id, id); } continue;}

						break (msg.code, msg.success);
					},
					Err(e) => elogln!("Warning: Received an invalid client message\n    {e:?}"),
				}
			},
			Err(e) => elogln!("Warning: Failed to open a received client connection\n    {e}"),
		}
	};

	let status: ExitStatus = child.wait().map_err(|e| -> ErrorReason { elogln!("Error: Child wait failed\n    {e}"); ErrorReason::ChildWaitFailure })?;

	if !status.success() { if cli.verbose { elogln!("Error: Terminal failed"); } return Err(ErrorReason::TerminalFailure); }

	if success { Ok(()) } else { Err(ErrorReason::ChildError) }
}