use std::fmt::Display;

use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Author {
	pub name: Name
}

impl Display for Author {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.name {
			Name::Full(full) => write!(f, "{} {}", full.first, full.last),
			Name::Combined(combined) => write!(f, "{combined}"),
		}
	}
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