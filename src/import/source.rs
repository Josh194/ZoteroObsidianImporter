use std::io;

use chrono::format::StrftimeItems;
use serde::Deserialize;

use crate::api::Author;

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceImport {
	pub id: i32,
	pub key: String,
	pub kind: String,
	pub title: String,
	pub note: Option<String>,
	pub date: String,
	pub url: Option<String>,
	pub authors: Vec<Author>,
	pub tags: Vec<Tag>,
	pub date_added: String,
	pub date_modified: String,
	pub path: String,
	pub citation_key: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tag {
	pub tag: String
}

#[allow(unused)]
pub struct MonthDate {
	pub year: i32,
	pub month: u8
}

impl SourceImport {
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

	pub fn primary_author(&self) -> &Author {
		self.authors.iter().next().unwrap()
	}

	pub fn short_name(&self) -> String {
		match &self.primary_author().name {
			crate::api::Name::Full(full) => format!("{} {}", full.last, self.year()),
			crate::api::Name::Combined(combined) => format!("{} {}", combined, self.year()),
		}
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