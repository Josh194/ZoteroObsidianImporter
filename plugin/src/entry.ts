import { Core } from './core'
import manifest from './manifest.json';

const version = manifest.version;

let core: Core | null;

export function install() {
	Zotero.log(`Installed ZOImporter ${version}`);
}

export async function startup({ id, version, rootURI }: {id: any, version: any, rootURI: any}) {
	Zotero.log(`Starting ZOImporter ${version}`);
	
	Zotero.PreferencePanes.register({
		pluginID: manifest.applications.zotero.id,
		src: rootURI + 'preferences.xhtml', // ? What is this?
		scripts: [rootURI + 'preferences.js'] // ? What is this?
	});

	core = new Core(id, version, rootURI);
	core.addToAllWindows();

	core.addImportMenu();
}

export function onMainWindowLoad({ window }: {window: any}) {
	core?.addToWindow(window);
}

export function onMainWindowUnload({ window }: {window: any}) {
	core?.removeFromWindow(window);
}

export function shutdown() {
	Zotero.log(`Shutting down ZOImporter ${version}`);

	core?.removeFromAllWindows();
	core = null;
}

export function uninstall() {
	Zotero.log(`Uninstalled ZOImporter ${version}`);
}
