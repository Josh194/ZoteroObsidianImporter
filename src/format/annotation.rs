use std::{fs::File, io::{self, Write}};

use serde::Serialize;

use crate::{document::annotation::ZAnnotation, import::source::DocumentMeta};

use super::NoteTarget;

#[derive(Debug, Clone)]
pub struct AnnotationImportData<'a> {
	pub source: &'a DocumentMeta,
	pub annot: ZAnnotation<'a>
}

#[derive(Debug, Clone)]
pub struct AnnnotationPersist {
	pub content_section: String
}

impl Default for AnnnotationPersist {
	fn default() -> Self {
		Self {
			content_section: "\n".to_owned()
		}
	}
}

pub type AnnotationTarget<'a> = NoteTarget<'a, AnnotationImportData<'a>, AnnnotationPersist>;

#[derive(Debug, Clone, Serialize)]
struct AnnotationProperties {
	source: String,
	tags: Vec<String>
}

static HEADER_NAMES: &[&str] = &["Summary", "Quotable", "Aim"];

pub fn fixup_headers(mut buffer: String) -> String {
	let names: Vec<String> = HEADER_NAMES.iter().map(|s| { "\n".to_owned() + s + "\n" }).collect();

	// * Check for any matches at the start of the buffer.
	for name in &names {
		if buffer.starts_with(&name[1..]) {
			buffer = "# ".to_owned() + &buffer;
			break;
		}
	}

	// * Replace general matches
	for name in names {
		let mut replacement = name.clone();
		replacement.insert_str(1, "# ");

		buffer = buffer.replace(&name, &replacement);
	}

	buffer
}

#[derive(Debug)]
pub enum AnnotationExportError {
	Io(io::Error),
	PropertyDeserialize(serde_yml::Error)
}

impl From<io::Error> for AnnotationExportError {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<serde_yml::Error> for AnnotationExportError {
	fn from(value: serde_yml::Error) -> Self {
		Self::PropertyDeserialize(value)
	}
}

pub fn write_annotation(target: AnnotationTarget) -> Result<(), AnnotationExportError> {
	let AnnotationTarget { file, data, persist } = target;
	let AnnotationImportData { source, annot } = data;

	let props = AnnotationProperties {
		source: format!("[[{}]]", source.file_name()),
		tags: annot.tags.iter().map(|s| { s.replace(" ", "_") }).collect()
	};

	let AnnnotationPersist { content_section: persist_sec } = persist.unwrap_or_default();

	let mut buffer = annot.base.content.as_ref().cloned().unwrap_or_default();
	buffer = fixup_headers(buffer);

	AnnotationNote {
		properties: &serde_yml::to_string(&props)?,
		persist: &persist_sec,
		content: &buffer
	}.write_to(file)?;

	Ok(())
}

struct AnnotationNote<'a> {
	properties: &'a str,
	persist: &'a str,
	content: &'a str
}

impl<'a> AnnotationNote<'a> {
	pub fn write_to(self, out: &mut File) -> Result<(), io::Error> {
		let Self {
			properties,
			persist,
			content
		} = self;

		out.write_all(format!("---\n{properties}---\n\n**Persistent Notes**\n\n---\n\n<!--SZO-Persist-Begin-->{persist}%%SZO-Persist-End%%\n\n---\n\n{content}").as_bytes())
	}
}