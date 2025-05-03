use serde::Deserialize;

use crate::document::annotation::Annotation;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportFile {
	pub version: i64,
	pub export: serde_json::Value
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
	pub annotations: Box<[Annotation]>
}