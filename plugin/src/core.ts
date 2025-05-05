import { perform_export } from "./export";
import { Util } from "./util";
import manifest from './manifest.json';

export class Core {
	id: null;
	version: null;
	rootURI: null;
	// children: string[]
	
	constructor(id: null, version: null, rootURI: null ) {
		this.id = id;
		this.version = version;
		this.rootURI = rootURI;
		// this.children = [];
	}

	addImportMenu() {
		// * This is automatically removed based on plugin id as per https://www.zotero.org/support/dev/zotero_7_for_developers#custom_reader_event_handlers.
		Zotero.Reader.registerEventListener(
			"createViewContextMenu",
			(event) => {
				let { reader, params, append } = event;

				append({
					label: "Run ZOImporter",

					async onCommand() {
						await clickHandler();
					},
				});
			},
			manifest.applications.zotero.id
		);
	}

	addToWindow(window: _ZoteroTypes.MainWindow) {}

	addToAllWindows() {
		var windows = Zotero.getMainWindows();
		
		for (let win of windows) {
			if (!win.ZoteroPane) continue;
			
			this.addToWindow(win);
		}
	}
	
	removeFromWindow(window: _ZoteroTypes.MainWindow) {}

	removeFromAllWindows() {
		var windows = Zotero.getMainWindows();
		
		for (let win of windows) {
			if (!win.ZoteroPane) continue;

			this.removeFromWindow(win);
		}
	}
}

async function clickHandler(): Promise<void> {
	Zotero.log("clicked");

	try {
		let result = await perform_export()

		if (result !== true) {
			Zotero.log("Export failed");
			Zotero.logError(result);
		}
	} catch (e) {
		if (e instanceof Util.ImporterNotFoundError) {
			Zotero.log("Importer not found");
		} else if (e instanceof Util.UnsupportedOSError) {
			Zotero.log("Unsupported OS");
		} else {
			throw e;
		}
	}
}