use std::{fmt::Display, fs::File, io::{self, Write}};

use serde::Serialize;

use crate::{command::import::ImportedAnnot, document::annotation::{self, ZAnnotation}, import::source::DocumentMeta};

use super::NoteTarget;

#[derive(Debug, Clone)]
pub struct AnnotationImportData<'a> {
	pub source: &'a DocumentMeta,
	pub export: Option<&'a ImportedAnnot>,
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
	let AnnotationImportData { source, export, annot } = data;

	let props = AnnotationProperties {
		source: format!("[[{}]]", source.file_name()),
		tags: annot.tags.iter().map(|s| { s.replace(" ", "_") }).collect()
	};

	let AnnnotationPersist { content_section: persist_sec } = persist.unwrap_or_default();

	let mut buffer = annot.base.content.as_ref().cloned().unwrap_or_default();
	buffer = fixup_headers(buffer);

	AnnotationNote {
		properties: &serde_yml::to_string(&props)?,
		text: export.map(|a| a.text.as_str()).unwrap_or("N/A"),
		colour: annot.base.colour.map(|c| c.into()),
		persist: &persist_sec,
		content: &buffer
	}.write_to(file)?;

	Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Colour {
	r: u8,
	g: u8,
	b: u8
}

impl From<annotation::Colour> for Colour {
	fn from(value: annotation::Colour) -> Self {
		Self { r: value.r, g: value.g, b: value.b }
	}
}

impl Display for Colour {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Self { r, g, b } = self;
		write!(f, "#{r:02x}{g:02x}{b:02x}")
	}
}

struct AnnotationNote<'a> {
	properties: &'a str,
	text: &'a str,
	colour: Option<Colour>,
	persist: &'a str,
	content: &'a str
}

impl<'a> AnnotationNote<'a> {
	pub fn write_to(self, out: &mut File) -> Result<(), io::Error> {
		let Self {
			properties,
			text,
			colour,
			persist,
			content
		} = self;

		// TODO: Improve this.
		// * Markdown does not affect the styled text in Obsidian, so we use HTML for the italics as well.
		let text: String = match colour {
			Some(c) => format!("<mark style=\"background-color: {c};\"><i>{text}</i></mark>"),
			None => format!("<i>{text}</i>"),
		};

		out.write_all(format!("---\n{properties}---\n\n\"{text}\"\n\n**Persistent Notes**\n\n---\n\n<!--SZO-Persist-Begin-->{persist}%%SZO-Persist-End%%\n\n---\n\n{content}").as_bytes())
	}
}