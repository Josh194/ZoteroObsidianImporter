use serde::Deserialize;

use super::{annotation::Annotation, source::SourceImport};

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
	pub source: SourceImport,
	pub annotations: Box<[Annotation]>
}