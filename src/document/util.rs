use core::str;

use lopdf::{Dictionary, ObjectId, Stream};

use crate::text::{self, ParseFailure};

pub enum ParseError {
	NameNotUTF8,
	BadStringFormat(ParseFailure)
}

impl From<ParseFailure> for ParseError {
	fn from(value: ParseFailure) -> Self {
		Self::BadStringFormat(value)
	}
}

pub enum Object<'a> {
	Null,
	Boolean(bool),
	Integer(i64),
	Real(f32),
	Name(String),
	String(String),
	Array(&'a Vec<lopdf::Object>),
	Dictionary(&'a Dictionary),
	Stream(&'a Stream),
	Reference(ObjectId)
}

impl<'a> TryFrom<&'a lopdf::Object> for Object<'a> {
	type Error = ParseError;

	fn try_from(value: &'a lopdf::Object) -> Result<Self, Self::Error> {
		Ok(match value {
			lopdf::Object::Null => Object::Null,
			lopdf::Object::Boolean(x) => Object::Boolean(*x),
			lopdf::Object::Integer(x) => Object::Integer(*x),
			lopdf::Object::Real(x) => Object::Real(*x),
			lopdf::Object::Name(x) => Object::Name(str::from_utf8(x).map_err(|_| ParseError::NameNotUTF8)?.to_owned()),
			lopdf::Object::String(x, _format) => Object::String(text::parse_string(x)?),
			lopdf::Object::Array(x) => Object::Array(x),
			lopdf::Object::Dictionary(x) => Object::Dictionary(x),
			lopdf::Object::Stream(x) => Object::Stream(x),
			lopdf::Object::Reference(x) => Object::Reference(*x),
		})
	}
}