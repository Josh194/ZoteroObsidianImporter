export namespace ZO {
	export class SelectionFile {
		version: number;
		selection: any;

		constructor(version: number) {
			this.version = version;
			this.selection = undefined;
		}
	}

	export class Selection {
		library_id: number;
		document_id: number;

		constructor(library_id: number, document_id: number) {
			this.library_id = library_id;
			this.document_id = document_id;
		}
	}
}