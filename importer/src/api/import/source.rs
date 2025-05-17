use std::io;

use chrono::format::StrftimeItems;
use serde::Deserialize;

use crate::{api::shared::{Author, Name}, util::human_date::{self, HumanDate}};

// ! TODO: Sanity check this (eg for non emptiness) on import.
#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceImport {
	pub library: i32,
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
	pub path: String
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tag {
	pub name: String
}

impl SourceImport {
	// TODO: Handle errors.
	pub fn parse_date(&self) -> Result<HumanDate, ()> {		
		Ok(HumanDate::parse(&self.date).map_err(|_| ())?)
	}

	pub fn year(&self) -> u32 {
		self.parse_date().unwrap().year
	}

	pub fn primary_author(&self) -> &Author {
		self.authors.iter().next().unwrap()
	}

	pub fn short_name(&self) -> String {
		match &self.primary_author().name {
			Name::Full(full) => format!("{} {}", full.last, self.year()),
			Name::Combined(combined) => format!("{} {}", combined, self.year()),
		}
	}

	pub fn file_name(&self) -> String {
		self.title.clone()
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