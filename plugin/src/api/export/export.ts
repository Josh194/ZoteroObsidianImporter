import { Util } from "../../util";
import { Annotation } from "./annotations";
import { Source } from "./source";

export class ExportFile {
	version: number;
	data: ZExport;

	constructor(version: number, z_export: ZExport) {
		this.version = version;
		this.data = z_export;
	}
}

export class ZExport {
	source: Source;
	annotations: Annotation[];

	constructor(
		source: Source,
		annotations: Annotation[]
	) {
		this.source = source;
		this.annotations = annotations;
	}

	static try_from(item: Zotero.Item): ZExport | null {
		// * Might be too restrictive?
		if (!item.isPDFAttachment()) { return null; }

		let source: Source | null = Source.try_from(item);
		if (source == null) { return null; }

		let annotations: Annotation[] = item.getAnnotations()
			.map(Annotation.try_from)
			.map(Util.require_defined);

		return new ZExport(
			source,
			annotations
		);
	}
}