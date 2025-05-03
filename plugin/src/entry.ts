import { Core } from './core'

let core: Core | null;

export function install() {
	Zotero.log("Installed ZO 2.0");
}

export async function startup({ id, version, rootURI }: {id: any, version: any, rootURI: any}) {
	Zotero.log("Starting ZO 2.0");
	
	Zotero.PreferencePanes.register({
		pluginID: 'zo-importer@example.com',
		src: rootURI + 'preferences.xhtml', // ? What is this?
		scripts: [rootURI + 'preferences.js'] // ? What is this?
	});

	core = new Core(id, version, rootURI);
	core.addToAllWindows();
	await core.main();
}

export function onMainWindowLoad({ window }: {window: any}) {
	core?.addToWindow(window);
}

export function onMainWindowUnload({ window }: {window: any}) {
	core?.removeFromWindow(window);
}

export function shutdown() {
	Zotero.log("Shutting down 2.0");
	core?.removeFromAllWindows();
	core = null;
}

export function uninstall() {
	Zotero.log("Uninstalled 2.0");
}
