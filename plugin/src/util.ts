import { index_name } from "./export";

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

	export async function exec_importer(): Promise<true | Error> {
		const file_name: string = "zo_import_run";
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

		return await Zotero.Utilities.Internal.exec(file, [dir.path, index_name])
	}

	export function require_defined<T>(val: T | undefined): T {
		if (val === undefined) {
			throw new TypeError("Value was undefined!");
		}

		return val;
	}
}