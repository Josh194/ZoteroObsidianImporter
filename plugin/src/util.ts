import { export_name, index_name, selection_name } from "./export";

export namespace Util {
	export async function get_data_dir(): Promise<nsIFile> {
		let dir: string = PathUtils.join(Zotero.DataDirectory.dir, "zo_import");
		
		await Zotero.File.createDirectoryIfMissingAsync(dir);

		return Zotero.File.pathToFile(dir);
	}

	export class UnsupportedOSError extends Error {
		os: string;

		constructor(msg: string, os?: string) {
			super(msg);

			if (os === undefined) {
				this.os = "Unknown";
			} else {
				this.os = os;
			}
		}
	}

	export enum OSType {
		Windows,
		Mac,
		Linux,
		Unknown
	}

	export function get_os(): OSType {
		switch (true) {
			case Zotero.isWin: return OSType.Windows;
			case Zotero.isMac: return OSType.Mac;
			case Zotero.isLinux: return OSType.Linux;
			default: return OSType.Unknown;
		}
	}

	export class ImporterNotFoundError extends Error {}

	type ExecStage = "select" | "import";

	export async function exec_importer(stage: ExecStage): Promise<true | Error> {
		const shim: string = "shim";
		const dir: nsIFile = await get_data_dir();

		let file: nsIFile;

		let ext: string = "";

		switch (get_os()) {
			case OSType.Windows: {
				ext = ".exe";
				break;
			}
			case OSType.Mac: {
				ext = "";
				break;
			}
			default: {
				throw new UnsupportedOSError("Unsupported OS", await Zotero.getOSVersion());
			}
		}

		file = Zotero.File.pathToFile(PathUtils.join(dir.path, `${shim}${ext}`))

		if (!file.exists()) { throw new ImporterNotFoundError("Importer not found"); }
		if (!file.isExecutable()) { throw new ImporterNotFoundError("Importer not executable"); }

		function get_cmd(stage: ExecStage): string[] {
			switch (stage) {
				case "select": return ["select", "-f", index_name, "-o", selection_name];
				case "import": return ["import", "-f", export_name, "-i", index_name];
			}
		}

		return await Zotero.Utilities.Internal.exec(file, ["--verbose", "--log", "--working-directory", dir.path, "--command", `./ZOImporter${ext}`].concat(get_cmd(stage)));
	}

	export function require_defined<T>(val: T | null): T {
		if (val === null) {
			throw new TypeError("Value was undefined!");
		}

		return val;
	}
}