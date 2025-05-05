use std::{fs::File, io::{self, Write}};

use serde::Serialize;

use crate::api::import::source::SourceImport;

use super::target::NoteTarget;

#[derive(Debug, Clone)]
pub struct SourceImportData<'a> {
	pub source: &'a SourceImport
}

#[derive(Debug, Clone)]
pub struct SourcePersist {
	pub content_section: String
}

impl Default for SourcePersist {
	fn default() -> Self {
		Self {
			content_section: "\n".to_owned()
		}
	}
}

pub type SourceTarget<'a> = NoteTarget<'a, SourceImportData<'a>, SourcePersist>;

#[derive(Debug, Clone, Serialize)]
pub struct SourceProperties {
	pub authors: Vec<String>,
	pub date: String,
	pub tags: Vec<String>
}

#[derive(Debug)]
pub enum SourceExportError {
	Io(io::Error),
	PropertyDeserialize(serde_yml::Error)
}

impl From<io::Error> for SourceExportError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<serde_yml::Error> for SourceExportError {
	fn from(value: serde_yml::Error) -> Self {
		Self::PropertyDeserialize(value)
	}
}

pub fn write_source(target: SourceTarget) -> Result<(), SourceExportError> {
	let SourceTarget { file, data, persist } = target;
	let SourceImportData { source } = data;

	let props = SourceProperties {
		authors: source.authors.iter().map(|a| a.to_string()).collect(),
		date: source.date.clone(),
		tags: source.tags.iter().map(|s| { s.tag.replace(" ", "_") }).collect()
	};

	let SourcePersist { content_section: persist_sec } = persist.unwrap_or_default();
	
	SourceNote {
		properties: &serde_yml::to_string(&props)?,
		key: &source.key,
		persist: &persist_sec,
		title: &source.title,
		content: source.note.as_ref().map(|s| s.as_str())
	}.write_to(file)?;

	Ok(())
}

struct SourceNote<'a> {
	properties: &'a str,
	key: &'a str,
	persist: &'a str,
	title: &'a str,
	content: Option<&'a str>
}

impl<'a> SourceNote<'a> {
	pub fn write_to(self, out: &mut File) -> Result<(), io::Error> {
		let Self {
			properties,
			key,
			persist,
			title,
			content
		} = self;

		let content = content.unwrap_or_default();

		out.write_all(format!("---\n{properties}---\n\n[Open in Zotero](zotero://select/library/items/{key})\n\n**Persistent Notes**\n\n---\n\n<!--SZO-Persist-Begin-->{persist}%%SZO-Persist-End%%\n\n# {title}\n\n---\n\n{content}").as_bytes())
	}
}