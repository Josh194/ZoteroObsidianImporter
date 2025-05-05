import { AuthorIndex } from "..";
import { Util } from "../../util";
import { Tag } from "./annotations";

export class Source {
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
	citation_key: string;

	constructor(
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
		path: string,
		citation_key: string
	) {
		this.id = id;
		this.key = key;
		this.kind = kind;
		this.title = title;
		if (note !== null) { this.note = note };
		this.date = date;
		if (url !== null) { this.url = url };
		this.authors = authors;
		this.tags = tags;
		this.date_added = date_added;
		this.date_modified = date_modified;
		this.path = path;
		this.citation_key = citation_key;
	}

	static try_from(item: Zotero.Item): Source | null {
		if (!item.isPDFAttachment()) { return null; }

		let path = item.getFilePath();
		if (path == false) { return null; }

		let parent = item.parentItem;
		if (parent == undefined) { return null; }

		return new Source(
			parent.id,
			parent.key,
			parent.itemType,
			parent.getDisplayTitle(),
			parent.getField("abstractNote"),
			parent.getField("date"),
			parent.getField("url"),
			parent.getCreatorsJSON().filter((c) => c.creatorType === "author").map(AuthorIndex.from),
			parent.getTags().map(Tag.from),
			Zotero.Date.sqlToISO8601(parent.dateAdded),
			Zotero.Date.sqlToISO8601(parent.dateModified),
			path,
			parent.getField("citationKey"),
		);
	}
}