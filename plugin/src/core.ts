export class Core {
	id: null;
	version: null;
	rootURI: null;
	children: string[]
	
	constructor(id: null, version: null, rootURI: null ) {
		this.id = id;
		this.version = version;
		this.rootURI = rootURI;
		this.children = [];
	}
	
	async main() {
		// Global properties are included automatically in Zotero 7
		var host = new URL('https://foo.com/path').host;
		Zotero.log(`Host is ${host}`);
		await this.test();
	}

	private async test() {
		let path = Zotero.File.pathToFile(Zotero.DataDirectory.dir);
		path.appendRelativePath("test.txt");
	
		if (Zotero.isWin) {
			let exe = Zotero.File.pathToFile(Zotero.DataDirectory.dir);
			exe.appendRelativePath("zo_import_run.bat");
	
			await Zotero.Utilities.Internal.exec(exe, [Zotero.File.pathToFile(Zotero.DataDirectory.dir).path])
		} else {
			throw new Error("Unsupported OS")
		}
	
		Zotero.log("Continuing");
	
		//new FileReader().readAsText(path);
		await Zotero.File.putContentsAsync(path, "hi");
		// let a = await Zotero.File.getContentsAsync(path);
	
		// switch (true) {
		// 	case typeof(a) === "string": {
		// 		Zotero.log(a);
		// 		break;
		// 	}
		// 	case typeof a !== 'object': {
		// 		Zotero.log("void");
		// 		break;
		// 	}
		// 	case a instanceof Uint8Array: {
		// 		Zotero.log(a);
		// 		break;
		// 	}
		// 	case !(a instanceof Uint8Array): {
		// 		let b = a;
		// 		Zotero.log(a);
		// 		break;
		// 	}
		// }
	
		const item = Zotero.Items.get(3);
		//Zotero.getZoteroPanes()[0]?.exportPDF(0);
		Zotero.log("hello");
		Zotero.log(item.annotationText);
	}

	addToWindow(window: _ZoteroTypes.MainWindow) {
		let doc = window.document;
		let menu = doc.createXULElement("button");
		menu.setAttribute("id", "zo_export_begin");
		menu.setAttribute("type", "button");
		menu.addEventListener("click", clickHandler);
	
		function isLabel(obj: object): obj is XUL.ILabel {
			if (!("label" in obj)) { return false; }
			return  typeof(obj.label) === "string";
		}
	
		function isDisabled(obj: object): obj is XUL.IDisabled {
			if (!("disabled" in obj)) { return false; }
			return  typeof(obj.disabled) === "boolean";
		}
	
		function isCheckboxElement(obj: XULElement): obj is XULCheckboxElement {
			if (!isLabel(obj) || !isDisabled(obj)) { return false; }
			if (!("checked" in obj)) { return false; }
			return typeof(obj.checked) === "boolean";
		}
	
		function assumeXUL(obj: any): obj is XULElement {
			return true;
		}

		function assumeButtonElement(obj: XULElement): obj is XULButtonElement {
			return true;
		}
	
		if (!assumeXUL(menu)) { throw Error("?") }
		if (isCheckboxElement(menu)) {
			Zotero.log("Is Checkbox");
		} else {
			Zotero.log("Is Not Checkbox");
		}
	
		doc.getElementById('menu_viewPopup')?.appendChild(menu);
		this.children.push(menu.id);
	}

	addToAllWindows() {
		var windows = Zotero.getMainWindows();
		for (let win of windows) {
			if (!win.ZoteroPane) continue;
			this.addToWindow(win);
		}
	}
	
	removeFromWindow(window: _ZoteroTypes.MainWindow) {
		var doc = window.document;
	
		for (let id of this.children) {
			doc.getElementById(id)?.remove();
		}
	}

	removeFromAllWindows() {
		var windows = Zotero.getMainWindows();
		
		for (let win of windows) {
			if (!win.ZoteroPane) continue;

			this.removeFromWindow(win);
		}
	}
}

function clickHandler(): void {
	Zotero.log("clicked");
}