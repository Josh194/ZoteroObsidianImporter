import { Util } from "../../util";

export class Annotation {
	key: string;
	kind: keyof typeof AnnotationType;
	page: number;
	text?: string;
	comment?: string;
	colour: string;
	date_added: string;
	date_modified: string;
	tags: Tag[];

	constructor(
		key: string,
		kind: keyof typeof AnnotationType,
		page: number,
		text: string | null,
		comment: string | null,
		colour: string,
		date_added: string,
		date_modified: string,
		tags: Tag[]
	) {
		this.key = key;
		this.kind = kind;
		this.page = page;
		if (text !== null) { this.text = text; };
		if (comment !== null) { this.comment = comment; };
		this.colour = colour;
		this.date_added = date_added;
		this.date_modified = date_modified;
		this.tags = tags;
	}

	static try_from(item: Zotero.Item): Annotation | null {
		if (!item.isAnnotation()) { return null; }

		type TypeMap = {
			[key in _ZoteroTypes.Annotations.AnnotationType]: keyof typeof AnnotationType;
		};

		const type_map: TypeMap = {
			highlight: "Highlight",
			image: "Unknown",
			ink: "Unknown",
			note: "Unknown",
			underline: "Unknown",
			text: "Unknown"
		};

		return new Annotation(
			item.key,
			type_map[item.annotationType],
			Number.parseInt(item.annotationPageLabel), // ! TODO: Parse position into a JSON object instead.
			item.annotationText,
			item.annotationComment,
			item.annotationColor,
			Zotero.Date.sqlToISO8601(item.dateAdded),
			Zotero.Date.sqlToISO8601(item.dateModified),
			item.getTags().map(Tag.from)
		);
	}
}

export enum AnnotationType {
	Highlight,
	Unknown
}

export class Tag {
	name: string;

	constructor(name: string) {
		this.name = name;
	}

	static from(tag: { tag: string; type?: number }): Tag {
		return new Tag(
			tag.tag
		);
	}
}

export class Colour {
	r: number;
	g: number;
	b: number;

	constructor(r: number, g: number, b: number) {
		this.r = r;
		this.g = g;
		this.b = b;
	}
}