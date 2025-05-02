import { LibraryIndex, ZIndex } from "./index";
import { Util } from "./util";

export const index_name: string = "index.json";

export async function perform_export(): Promise<true | Error> {
	Zotero.log("Beginning ZO export");
	let index_file = PathUtils.join((await Util.get_data_dir()).path, index_name);

	await Zotero.File.putContentsAsync(index_file, JSON.stringify(new ZIndex(
		await Promise.all(Zotero.Libraries.getAll().map(LibraryIndex.from))
	)));

	return await Util.exec_importer();
}