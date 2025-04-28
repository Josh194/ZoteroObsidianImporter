use std::{fs::File, io::{self, Write}};

use serde::Serialize;

use crate::import::source::DocumentMeta;

use super::NoteTarget;

#[derive(Debug, Clone)]
pub struct SourceImportData<'a> {
	pub source: &'a DocumentMeta
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
		authors: source.creators.iter().filter(|c| c.creator_type == "author").map(|c| { format!("{} {}", c.first_name, c.last_name) }).collect(),
		date: source.date.clone(),
		tags: source.tags.iter().map(|s| { s.tag.replace(" ", "_") }).collect()
	};

	let SourcePersist { content_section: persist_sec } = persist.unwrap_or_default();
	
	SourceNote {
		properties: &serde_yml::to_string(&props)?,
		zotero: &source.select,
		persist: &persist_sec,
		title: &source.title,
		content: &source.abstract_note
	}.write_to(file)?;

	Ok(())
}

struct SourceNote<'a> {
	properties: &'a str,
	zotero: &'a str,
	persist: &'a str,
	title: &'a str,
	content: &'a str
}

impl<'a> SourceNote<'a> {
	pub fn write_to(self, out: &mut File) -> Result<(), io::Error> {
		let Self {
			properties,
			zotero,
			persist,
			title,
			content
		} = self;

		out.write_all(format!("---\n{properties}---\n\n[Open in Zotero]({zotero})\n\n**Persistent Notes**\n\n---\n\n<!--SZO-Persist-Begin-->{persist}%%SZO-Persist-End%%\n\n# {title}\n\n---\n\n{content}").as_bytes())
	}
}