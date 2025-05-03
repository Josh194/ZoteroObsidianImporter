import { Util } from "../../util";
import { Annotation } from "./annotations";

export class ExportFile {
	version: number;
	export: ZExport;

	constructor(version: number, z_export: ZExport) {
		this.version = version;
		this.export = z_export;
	}
}

export class ZExport {
	annotations: Annotation[];

	constructor(annotations: Annotation[]) {
		this.annotations = annotations;
	}

	static try_from(item: Zotero.Item): ZExport | null {
		// * Might be too restrictive?
		if (!item.isPDFAttachment()) { return null; }
		
		let annotations: Annotation[] = item.getAnnotations()
			.map(Annotation.try_from)
			.map(Util.require_defined);
		
		return new ZExport(
			annotations
		);	
	}
}