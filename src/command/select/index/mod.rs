use serde::Deserialize;

pub mod util;

// * Currently can't (or at least don't want to) use `#[serde(flatten)]` to handle the plugin's `IndexBase` inheritance due to issues around `#[serde(deny_unknown_fields)]`.
// * Look at this again in the future if the issue is ever solved.

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
	pub version: i64,
	pub index: serde_json::Value
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
	pub libraries: Box<[Library]>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
	pub id: i64,
	pub name: String,
	pub documents: Box<[Document]>,
	pub collections: Box<[Collection]>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
	pub id: i64,
	pub title: String,
	pub creators: Box<[Author]>,
	pub collection_ids: Box<[i64]>,
	pub date_added: String,
	pub date_modified: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
	pub name: Name
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "format", content = "value")]
#[serde(deny_unknown_fields)]
pub enum Name {
	#[serde(rename = "full")] Full(FullName),
	#[serde(rename = "combined")] Combined(String)
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FullName {
	pub first: String,
	pub last: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
	pub id: i64,
	pub name: String,
	pub document_ids: Box<[i64]>,
	pub collections: Box<[Collection]>
}