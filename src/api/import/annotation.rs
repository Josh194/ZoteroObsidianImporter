use std::fmt::{self, Display};

use chrono::{DateTime, FixedOffset};
use serde::{de::{self, Visitor}, Deserialize};

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
	pub key: String,
	pub kind: AnnotationType,
	pub page: u32,
	pub text: Option<String>,
	pub comment: Option<String>,
	pub colour: Colour,
	pub date_added: DateTime<FixedOffset>,
	pub date_modified: DateTime<FixedOffset>,
	pub tags: Box<[Tag]>
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum AnnotationType {
	Highlight,
	#[serde(untagged)]
	Unknown
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tag {
	pub name: String
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct Colour {
	pub r: u8,
	pub g: u8,
	pub b: u8
}

impl Display for Colour {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Self { r, g, b } = self;
		write!(f, "#{r:02x}{g:02x}{b:02x}")
	}
}

impl<'de> Deserialize<'de> for Colour {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_str(ColourVisitor)
	}
}

struct ColourVisitor;

impl<'de> Visitor<'de> for ColourVisitor {
	type Value = Colour;

	fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "a color string represented as a hexadecimal RGB value with a leading #")
	}

	fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
		let chars: Box<[char]> = v.chars().collect();
		let arr: [char; 7] = chars.as_ref().try_into().map_err(|_| E::invalid_length(chars.len(), &"5"))?;

		if arr[0] != '#' { return Err(E::invalid_value(de::Unexpected::Char(arr[0]), &"#")); }

		let [_, r, g, b] = u32::from_str_radix(&v[1..], 16).map_err(|e| E::custom(e))?.to_be_bytes();

		Ok(Colour { r, g, b })
	}
}