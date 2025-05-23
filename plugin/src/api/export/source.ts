import { AuthorIndex } from "..";
import { Util } from "../../util";
import { Tag } from "./annotations";

export class Source {
	library: number;
	id: number;
	key: string;
	kind: string;
	title: string;
	note?: string;
	date: string;
	url?: string;
	authors: AuthorIndex[];
	tags: Tag[];
	date_added: string;
	date_modified: string;
	path: string;

	constructor(
		library: number,
		id: number,
		key: string,
		kind: string,
		title: string,
		note: string | null,
		date: string,
		url: string | null,
		authors: AuthorIndex[],
		tags: Tag[],
		date_added: string,
		date_modified: string,
		path: string
	) {
		this.library = library;
		this.id = id;
		this.key = key;
		this.kind = kind;
		this.title = title;
		if (note !== null) { this.note = note; };
		this.date = date;
		if (url !== null) { this.url = url; };
		this.authors = authors;
		this.tags = tags;
		this.date_added = date_added;
		this.date_modified = date_modified;
		this.path = path;
	}

	static try_from(item: Zotero.Item): Source | null {
		if (!item.isPDFAttachment()) { return null; }

		let path = item.getFilePath();
		if (path == false) { return null; }

		let parent = item.parentItem;
		if (parent == undefined) { return null; }

		let title: string = parent.getField("shortTitle");
		if (title === "") { title = parent.getDisplayTitle(); }

		// TODO: Check all fields for non-emptiness.

		return new Source(
			parent.libraryID,
			parent.id,
			parent.key,
			parent.itemType,
			title,
			parent.getField("abstractNote"),
			parent.getField("date"),
			parent.getField("url"),
			parent.getCreatorsJSON().filter((c) => c.creatorType === "author").map(AuthorIndex.from),
			parent.getTags().map(Tag.from),
			Zotero.Date.sqlToISO8601(parent.dateAdded),
			Zotero.Date.sqlToISO8601(parent.dateModified),
			path
		);
	}
}