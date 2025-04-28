use std::{fs, io, path::Path};

use chrono::format::StrftimeItems;
use serde::Deserialize;
use serde_json::Value;

use crate::config::IMPORT_META_NAME;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct ImportVersionMeta {
	zotero: String,
	bbt: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct ImportMeta {
	config: Value,
	version: ImportVersionMeta,
	collections: Value, // TODO: Figure out what this format is.
	items: Vec<serde_json::Map<String, Value>>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct Creator {
	#[serde(rename = "firstName")] pub first_name: String,
	#[serde(rename = "lastName")] pub last_name: String,
	#[serde(rename = "creatorType")] pub creator_type: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
	pub tag: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct Attachment {
	#[serde(rename = "itemType")] pub item_type: String,
	pub title: String,
	pub tags: Vec<Tag>, // TODO: Is this actually the same as the `DocumentMeta` tag type?
	pub relations: Value, // TODO: Figure out what this format is.
	#[serde(rename = "dateAdded")] pub date_added: String,
	#[serde(rename = "dateModified")] pub date_modified: String,
	pub uri: String,
	pub path: String,
	pub select: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct DocumentMeta {
	pub version: i32,
	#[serde(rename = "itemType")] pub item_type: String,
	pub title: String,
	#[serde(rename = "abstractNote")] pub abstract_note: String,
	pub date: String,
	pub language: String,
	pub url: String,
	#[serde(rename = "accessDate")] pub access_date: String,
	pub creators: Vec<Creator>,
	pub tags: Vec<Tag>,
	pub relations: Value, // TODO: Figure out what this format is.
	#[serde(rename = "dateAdded")] pub date_added: String,
	#[serde(rename = "dateModified")] pub date_modified: String,
	pub uri: String,
	pub attachments: Vec<Attachment>,
	pub notes: Vec<Value>, // TODO: Figure out what this format is.
	#[serde(rename = "citationKey")] pub citation_key: String,
	#[serde(rename = "itemID")] pub item_id: i32,
	#[serde(rename = "itemKey")] pub item_key: String,
	#[serde(rename = "libraryID")] pub library_id: i32,
	pub select: String
}

#[allow(unused)]
pub struct MonthDate {
	pub year: i32,
	pub month: u8
}

impl DocumentMeta {
	pub fn parse_date(&self) -> Result<MonthDate, chrono::ParseError> {
		let mut parsed = chrono::format::Parsed::new();
		
		chrono::format::parse(&mut parsed, &self.date, StrftimeItems::new("%B %Y"))?;

		Ok(MonthDate {
			year: parsed.year().unwrap(),
			month: parsed.month().unwrap() as u8,
		})
	}

	pub fn year(&self) -> i32 {
		self.parse_date().unwrap().year
	}

	pub fn primary_author(&self) -> &Creator {
		self.creators.iter().filter(|c| c.creator_type == "author").next().unwrap()
	}

	pub fn short_name(&self) -> String {
		format!("{} {}", self.primary_author().last_name, self.year())
	}

	pub fn file_name(&self) -> String {
		self.citation_key.clone()
	}
}

#[derive(Debug)]
pub enum ImportSourceError {
	WrongItemCount,
	InvalidItemFormat,
	InvalidJson(serde_path_to_error::Error<serde_json::Error>),
	Filesystem(io::Error)
}

impl From<serde_path_to_error::Error<serde_json::Error>> for ImportSourceError {
	fn from(value: serde_path_to_error::Error<serde_json::Error>) -> Self {
		Self::InvalidJson(value)
	}
}

impl From<io::Error> for ImportSourceError {
	fn from(value: io::Error) -> Self {
		Self::Filesystem(value)
	}
}

pub fn import_source<P: AsRef<Path>>(path: P) -> Result<DocumentMeta, ImportSourceError> {
	let data: String = fs::read_to_string(path.as_ref().join(IMPORT_META_NAME))?;

	let import: ImportMeta = serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&data))?;

	if import.items.len() != 1 { return Err(ImportSourceError::WrongItemCount); }

	let item = &import.items[0];

	let Some(Value::String(kind)) = item.get("itemType") else { return Err(ImportSourceError::InvalidItemFormat) };
	if kind != "document" { return Err(ImportSourceError::WrongItemCount); }

	Ok(serde_path_to_error::deserialize(item)?)
}