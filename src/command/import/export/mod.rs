use serde::Deserialize;

use crate::document::annotation::{Annotation, Colour, Point, Rectangle, ZAnnotation};

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
	pub version: i64,
	pub export: serde_json::Value
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotations {
	pub annotations: Box<[ZAnnotationNew]>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
// TODO: Re-enable this.
// #[serde(deny_unknown_fields)]
pub struct ZAnnotationNew {
	pub key: String,
	#[serde(rename = "annotationType")] pub kind: String,
	#[serde(rename = "annotationText")] pub text: String,
	#[serde(rename = "annotationComment")] pub comment: String,
	#[serde(rename = "annotationColor")] pub colour: String,
	pub tags: Box<[Tag]>
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
	pub tag: String
}

fn parse_colour(s: &str) -> Colour {
	let int = u32::from_str_radix(s.strip_prefix("#").unwrap(), 16).unwrap();
	Colour { r: ((int >> 16) & 0xff) as u8, g: ((int >> 8) & 0xff) as u8, b: (int & 0xff) as u8 }
}

impl ZAnnotationNew {
	#[deprecated]
	pub fn make_annot(&self) -> Annotation {
		Annotation {
			subtype: crate::document::annotation::AnnotationType::Highlight,
			bounds: Rectangle { start: Point { x: 0.0, y: 0.0 }, end: Point { x: 0.0, y: 0.0 } },
			content: Some(self.comment.clone()),
			name: None,
			title: None, 
			colour: Some(parse_colour(&self.colour)),
			date: None,
			properties: lopdf::Dictionary::new()
		}
	}

	#[deprecated]
	pub fn make_z_annot<'a>(&self, annot: &'a Annotation) -> ZAnnotation<'a> {
		ZAnnotation { base: annot, tags: self.tags.iter().map(|t| t.tag.clone()).collect(), key: self.key.clone() }
	}
}