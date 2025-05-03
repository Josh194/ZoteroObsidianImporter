import { export_name, index_name } from "./export";

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
		function get_stage_file(stage: ExecStage): string {
			switch (stage) {
				case "select": return "zo_select_run";
				case "import": return "zo_import_run";
			}
		}

		const file_name: string = get_stage_file(stage);

		const dir: nsIFile = await get_data_dir();

		let file: nsIFile;

		switch (get_os()) {
			case OSType.Windows: {
				file = Zotero.File.pathToFile(PathUtils.join(dir.path, `${file_name}.bat`));
				break;
			}
			default: {
				throw new UnsupportedOSError("Unsupported OS", await Zotero.getOSVersion());
			}
		}

		if (!file.exists() || !file.isExecutable()) { throw new ImporterNotFoundError("Importer not found"); }

		function get_args(stage: ExecStage): string[] {
			switch (stage) {
				case "select": return [index_name];
				case "import": return [export_name];
			}
		}

		return await Zotero.Utilities.Internal.exec(file, [dir.path, ])
	}

	export function require_defined<T>(val: T | null): T {
		if (val === null) {
			throw new TypeError("Value was undefined!");
		}

		return val;
	}
}