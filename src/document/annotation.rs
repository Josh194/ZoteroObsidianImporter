use std::str;

use lopdf::Dictionary;

use crate::text;

use super::doc::{AnnotationItem, BadFormatCause, DocParseError, DocumentItem, InvalidFormatError};

#[derive(Debug, Clone, Copy)]
pub struct Point {
	x: f32,
	y: f32
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
	start: Point,
	end: Point
}

#[derive(Debug, Clone, Copy)]
pub struct Colour {
	r: u8,
	g: u8,
	b: u8
}

#[derive(Debug, Clone, Copy)]
pub struct Date {
	year: i32,
	month: i8,
	day: i8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationType {
	Text,
	Link,
	Highlight,
	Popup,
	Attachment,
	Unknown
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Annotation {
	/// The type of the annotation.
	pub subtype: AnnotationType,
	/// The page bounds.
	pub bounds: Rectangle,
	/// The text content.
	pub content: Option<String>,
	/// The unique name/id.
	pub name: Option<String>,
	pub title: Option<String>,
	pub colour: Option<Colour>,
	/// The last modification date.
	pub date: Option<Date>,
	/// The full set of properties the annotation was derived from.
	pub properties: Dictionary
}

/// Checks that the `Type` field of an annotation either does not exist or has the value `"Annot"`.
fn check_type_valid(annotation: &Dictionary) -> Result<(), DocParseError> {
	if let Ok(ty) = annotation.get("Type".as_bytes()) {
		let Ok(ty) = ty.as_name() else {
			return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation(AnnotationItem::Type), reason: BadFormatCause::IncorrectType }));
		};
		
		if ty != "Annot".as_bytes() { return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation(AnnotationItem::Type), reason: BadFormatCause::IncorrectValue })) }
	}

	Ok(())
}

fn get_subtype(annotation: &Dictionary) -> Result<AnnotationType, DocParseError> {
	let Ok(subtype) = annotation.get("Subtype".as_bytes()) else {
		return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation(AnnotationItem::Subtype), reason: BadFormatCause::MissingRequired }));
	};

	let Ok(subtype) = subtype.as_name() else {
		return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation(AnnotationItem::Subtype), reason: BadFormatCause::IncorrectType }));
	};

	let Ok(string) = str::from_utf8(subtype) else {
		return Ok(AnnotationType::Unknown)
	};

	Ok(match string {
		"Text" => AnnotationType::Text,
		"Link" => AnnotationType::Link,
		"Highlight" => AnnotationType::Highlight,
		"Popup" => AnnotationType::Popup,
		"FileAttachment" => AnnotationType::Attachment,
		_ => AnnotationType::Unknown
	})
}

macro_rules! get_typed {
	($annotation:ident.get($key:literal as $ty:ident), $context:expr) => {
		(|| -> Result<Option<()>, DocParseError> {
			use crate::document::util::Object;

			$annotation as &lopdf::Dictionary;
			$key as &str;
			$context as AnnotationItem;

			let Ok(contents) = $annotation.get($key.as_bytes()) else { return Ok(None); };

			let lopdf::Object::$ty(contents) = contents else {
				return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation($context), reason: BadFormatCause::IncorrectType }));
			};
			todo!()
		})()
	};
}

fn a(annotation: &Dictionary) -> Result<Rectangle, DocParseError> {
	let _a = get_typed!(annotation.get("hello" as Boolean), AnnotationItem::Type);
	todo!()
}

// TODO
fn get_bounds(annotation: &Dictionary) -> Result<Rectangle, DocParseError> {
	Ok(Rectangle { start: Point { x: 0.0, y: 0.0 }, end: Point { x: 0.0, y: 0.0 } })
}

fn get_contents(annotation: &Dictionary) -> Result<Option<String>, DocParseError> {
	let Ok(contents) = annotation.get("Contents".as_bytes()) else { return Ok(None); };

	let Ok(contents) = contents.as_str() else {
		return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation(AnnotationItem::Contents), reason: BadFormatCause::IncorrectType }));
	};

	Ok(Some(text::parse_string(contents).or(Err(DocParseError::BadStringFormat))?))
}

fn get_name(annotation: &Dictionary) -> Result<Option<String>, DocParseError> {
	let Ok(name) = annotation.get("NM".as_bytes()) else { return Ok(None); };

	let Ok(name) = name.as_str() else {
		return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation(AnnotationItem::Name), reason: BadFormatCause::IncorrectType }));
	};

	Ok(Some(text::parse_string(name).or(Err(DocParseError::BadStringFormat))?))
}

fn get_title(annotation: &Dictionary) -> Result<Option<String>, DocParseError> {
	let Ok(title) = annotation.get("T".as_bytes()) else { return Ok(None); };

	let Ok(title) = title.as_str() else {
		return Err(DocParseError::Invalid(InvalidFormatError { item: DocumentItem::Annotation(AnnotationItem::Title), reason: BadFormatCause::IncorrectType }));
	};

	Ok(Some(text::parse_string(title).or(Err(DocParseError::BadStringFormat))?))
}

fn get_colour(annotation: &Dictionary) -> Result<Option<Colour>, DocParseError> {
	Ok(None)
}

fn get_date(annotation: &Dictionary) -> Result<Option<Date>, DocParseError> {
	Ok(None)
}

impl TryFrom<Dictionary> for Annotation {
	type Error = DocParseError;

	fn try_from(value: Dictionary) -> Result<Self, Self::Error> {
		check_type_valid(&value)?;

		Ok(Annotation {
			subtype: get_subtype(&value)?,
			bounds: get_bounds(&value)?,
			content: get_contents(&value)?,
			name: get_name(&value)?,
			title: get_title(&value)?,
			colour: get_colour(&value)?,
			date: get_date(&value)?,
			properties: value
		})
	}
}

#[derive(Debug, Clone)]
pub struct ZAnnotation<'a> {
	pub base: &'a Annotation,
	pub tags: Vec<String>,
	pub key: String
}

#[derive(Debug, Clone, Copy)]
pub enum ZDocParseError {
	NoKey,
	BadKeyType,
	NoTags,
	BadTagsType,
	BadString
}

fn get_key(annotation: &Dictionary) -> Result<String, ZDocParseError> {
	// TODO
	let Ok(contents) = annotation.get("Zotero:Key".as_bytes()) else { return Err(ZDocParseError::NoKey); };

	let Ok(contents) = contents.as_str() else {
		return Err(ZDocParseError::BadKeyType); // TODO
	};

	Ok(text::parse_string(contents).or(Err(ZDocParseError::BadString))?)
}

fn get_tags(annotation: &Dictionary) -> Result<Vec<String>, ZDocParseError> {
	// TODO
	let Ok(contents) = annotation.get("Zotero:Tags".as_bytes()) else { return Ok(Vec::new()); };

	let Ok(contents) = contents.as_str() else {
		return Err(ZDocParseError::BadTagsType); // TODO
	};

	let string = text::parse_string(contents).or(Err(ZDocParseError::BadString))?;

	let inner = string.strip_prefix('[').unwrap().strip_suffix(']').unwrap();

	Ok(inner.split(',').map(|s| {
		s.strip_prefix('\"').unwrap().strip_suffix('\"').unwrap().to_owned()
	}).collect())
}

impl<'a> TryFrom<&'a Annotation> for ZAnnotation<'a> {
	type Error = ZDocParseError;

	fn try_from(value: &'a Annotation) -> Result<Self, Self::Error> {
		Ok(ZAnnotation {
			base: value,
			tags: get_tags(&value.properties)?,
			key: get_key(&value.properties)?
		})
	}
}