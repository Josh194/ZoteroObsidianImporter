import { LibraryIndex, ZIndex } from "./index";
import type { ZO } from "./selection/selection";
import { Util } from "./util";

export const index_name: string = "index.json";
export const export_name: string = "export.json";
export const selection_name: string = "select.json";

const api_version: number = 1;

export async function perform_export(): Promise<true | Error> {
	Zotero.log("Beginning ZO export");
	let index_file = PathUtils.join((await Util.get_data_dir()).path, index_name);

	await Zotero.File.putContentsAsync(index_file, JSON.stringify(new ZIndex(
		api_version,
		await Promise.all(Zotero.Libraries.getAll().map(LibraryIndex.from))
	)));

	let select_result = await Util.exec_importer("select");

	if (select_result !== true) {
		return select_result;
	}

	let select_str = await Zotero.File.getContentsAsync(PathUtils.join((await Util.get_data_dir()).path, selection_name));

	if (typeof select_str !== "string") {
		throw new Error("Expected string from file");
	}

	// ! TODO: Typecheck this (currently `any`).
	let select_file: ZO.SelectionFile = JSON.parse(select_str);

	if (select_file.version !== api_version) {
		return new Error("Unsupported API version");
	}

	// ! TODO: Typecheck this (currently `any`).
	let selection: ZO.Selection = select_file.selection;

	// let library = Zotero.Libraries.get(selection.library_id);

	// if (library === false) {
	// 	throw new Error("Library not found");
	// }

	// !
	let attachment = Zotero.Items.get(selection.document_id).getAttachments()[0] as number;
	let annotations = Zotero.Items.get(attachment).getAnnotations();

	let export_file = PathUtils.join((await Util.get_data_dir()).path, export_name);

	await Zotero.File.putContentsAsync(export_file, JSON.stringify({
		version: 1,
		export: {
			annotations
		}
	}));

	return await Util.exec_importer("import");
}