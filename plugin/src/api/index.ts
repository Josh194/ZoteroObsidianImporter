import { Util } from "../util";

export class ZIndex {
	version: number;
	data: { libraries: LibraryIndex[] };

	constructor(version: number, libraries: LibraryIndex[]) {
		this.version = version;
		this.data = { libraries };
	}
}

class IndexBase {
	id: number;

	constructor(id: number) {
		this.id = id;
	}
}

export class LibraryIndex extends IndexBase {
	name: string;
	documents: DocumentIndex[];
	collections: CollectionIndex[];

	constructor(id: number, name: string, documents: DocumentIndex[], collections: CollectionIndex[]) {
		super(id);

		this.name = name;
		this.documents = documents;
		this.collections = collections;
	}

	static async from(library: Zotero.Library): Promise<LibraryIndex> {
		let documents: DocumentIndex[] = (await Zotero.Items.getAll(library.id, true))
			.filter((i) => i.itemType === "document") // ! TODO: This is wrong! Need to support other types, like "journalArticle", etc.
			.map(DocumentIndex.try_from)
			.map(Util.require_defined);

		return new LibraryIndex(
			library.id,
			library.name,
			documents,
			Zotero.Collections.getByLibrary(library.id).map(CollectionIndex.from)
		);
	}
}

export class DocumentIndex extends IndexBase {
	title: string;
	authors: AuthorIndex[];
	collection_ids: number[];
	date_added: string;
	date_modified: string;

	constructor(
		id: number,
		title: string,
		authors: AuthorIndex[],
		collection_ids: number[],
		date_added: string,
		date_modified: string
	) {
		super(id);

		this.title = title;
		this.authors = authors;
		this.collection_ids = collection_ids;
		this.date_added = date_added;
		this.date_modified = date_modified;
	}

	static try_from(item: Zotero.Item): DocumentIndex | null {
		let title: string = item.getField("shortTitle");
		if (title === "") { title = item.getDisplayTitle(); }

		return new DocumentIndex(
			item.id,
			title,
			// Use the JSON variant since the underlying function is currently difficult to properly typecheck.
			item.getCreatorsJSON().filter((c) => c.creatorType === "author").map(AuthorIndex.from),
			item.getCollections(),
			item.dateAdded,
			item.dateModified
		);
	}
}

type AuthorName = { format: string, value: string | { first: string, last: string } };

export class AuthorIndex {
	name: AuthorName;

	constructor(name: AuthorName) {
		this.name = name;
	}

	static from(creator: _ZoteroTypes.Item.CreatorJSON): AuthorIndex {
		let format: string;
		let name: string | { first: string, last: string };

		if (creator.name === undefined) {
			format = "full";
			if (creator.firstName === undefined || creator.lastName === undefined ) { throw new TypeError("Invalid creator JSON!"); }
			name = { first: creator.firstName, last: creator.lastName } ;
		} else {
			format = "combined";
			name = creator.name;
		}

		return new AuthorIndex(
			{ format, value: name }
		);
	}
}

export class CollectionIndex extends IndexBase {
	name: string;
	collections: CollectionIndex[];
	document_ids: number[];

	constructor(id: number, name: string, collections: CollectionIndex[], document_ids: number[]) {
		super(id);

		this.name = name;
		this.collections = collections;
		this.document_ids = document_ids;
	}

	static from(collection: Zotero.Collection): CollectionIndex {
		return new CollectionIndex(
			collection.id,
			collection.name,
			collection.getChildCollections().map(CollectionIndex.from),
			collection.getChildItems().filter((i) => i.itemType === "document").map((i) => i.id)
		);
	}
}