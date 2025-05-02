use std::{collections::BTreeMap, fmt::Display, path::{Path, PathBuf}};

use itertools::Itertools;

use crate::util::path::is_path_segment;

use super::{Collection, Library};

#[derive(Debug, Clone)]
pub struct LibraryCache<'a> {
	collection_map: BTreeMap<i64, CollectionCacheEntry<'a>>
}

#[derive(Debug, Clone, Copy)]
pub struct BoundCollectionCacheEntry<'a> {
	cache: &'a LibraryCache<'a>,
	pub entry: &'a CollectionCacheEntry<'a>
}

#[derive(Debug, Clone)]
pub struct CollectionCacheEntry<'a> {
	pub id: i64, // Useful when looking at parent entries.
	pub parent: Option<i64>,
	pub collection: &'a Collection
}

#[derive(Debug, Clone)]
pub struct CollectionPath<'a> {
	cache: &'a LibraryCache<'a>,
	path: Vec<i64>
}

impl<'a> CollectionPath<'a> {
	pub fn new(library: &'a LibraryCache<'a>) -> Self {
		Self { cache: library, path: Vec::new() }
	}

	pub fn push(&mut self, value: i64) {
		self.path.push(value);
	}
}

pub enum CollectionFilePathError {
	DangerousSegmentName
}

impl<'a> TryFrom<CollectionPath<'a>> for PathBuf {
	type Error = CollectionFilePathError;

	fn try_from(value: CollectionPath<'a>) -> Result<Self, Self::Error> {
		let mut out = PathBuf::new();

		for segment in value.path {
			let name = &value.cache.get_collection_raw(segment).unwrap().collection.name;

			if !is_path_segment(name) { return Err(CollectionFilePathError::DangerousSegmentName) }
			
			out.push(Path::new(name));
		}

		Ok(out)
	}
}

impl<'a> Display for CollectionPath<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.path.iter().map(|id| { &self.cache.get_collection_raw(*id).unwrap().collection.name }).join(" > "))
	}
}

impl<'a> AsRef<Collection> for CollectionCacheEntry<'a> {
	fn as_ref(&self) -> &Collection {
		self.collection
	}
}

impl<'a> BoundCollectionCacheEntry<'a> {
	unsafe fn new(cache: &'a LibraryCache, entry: &'a CollectionCacheEntry<'a>) -> Self {
		Self { cache, entry }
	}

	pub fn get_path(&self) -> CollectionPath<'a> {
		let Self { cache, entry } = self;

		let mut path = entry.parent.map(|id| cache.get_collection(id).unwrap().get_path()).unwrap_or_else(|| CollectionPath::new(cache));
		path.push(entry.id);

		path
	}
}

impl<'a> AsRef<Collection> for BoundCollectionCacheEntry<'a> {
	fn as_ref(&self) -> &Collection {
		self.entry.as_ref()
	}
}

pub enum LibraryIndexFormatError {
	DuplicateIds
}


impl<'a> LibraryCache<'a> {
	pub fn new(library: &'a Library) -> Result<Self, LibraryIndexFormatError> {
		let mut out = Self {
			collection_map: BTreeMap::new()
		};

		out.extend_collections(None, &library.collections)?;

		Ok(out)
	}

	fn extend_collections<Iter: IntoIterator<Item = &'a Collection>>(&mut self, parent: Option<i64>, collections: Iter) -> Result<(), LibraryIndexFormatError> {
		for collection in collections.into_iter() {
			let id = collection.id;

			if self.collection_map.insert(id, CollectionCacheEntry { id, parent, collection }).is_some() {
				return Err(LibraryIndexFormatError::DuplicateIds);
			}

			self.extend_collections(Some(id), &collection.collections)?;
		};

		Ok(())
	}

	pub fn get_collection_raw(&self, id: i64) -> Option<&CollectionCacheEntry> {
		self.collection_map.get(&id)
	}

	pub fn get_collection(&self, id: i64) -> Option<BoundCollectionCacheEntry> {
		self.get_collection_raw(id).map(|entry| unsafe { BoundCollectionCacheEntry::new(self, entry) })
	}
}