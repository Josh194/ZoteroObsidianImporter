use super::annotation::Annotation;

#[derive(Debug, Clone, Copy)]
pub enum AccessTarget {
	Annotations
}

#[derive(Debug, Clone, Copy)]
pub enum BadFormatCause {
	MissingRequired,
	IncorrectType,
	IncorrectValue
}

#[derive(Debug, Clone)]
pub struct InvalidFormatError {
	pub item: DocumentItem,
	pub reason: BadFormatCause
}

#[derive(Debug, Clone, Copy)]
pub enum DocumentItem {
	Annotation(AnnotationItem)
}

#[derive(Debug, Clone, Copy)]
pub enum AnnotationItem {
	Type,
	Subtype,
	Contents,
	Name,
	Title,
	Colour,
	Date
}

#[derive(Debug, Clone)]
pub enum DocParseError {
	/// The pdf is improperly formatted.
	Invalid(InvalidFormatError),
	/// Failed to access an internal value.
	AccessError(AccessTarget),
	/// Unable to parse a string.
	BadStringFormat
}

pub struct Document {
	pub annotations: Vec<Annotation>
}

impl TryFrom<lopdf::Document> for Document {
	type Error = DocParseError;

	fn try_from(doc: lopdf::Document) -> Result<Self, Self::Error> {
		let mut annotations: Vec<Annotation> = Vec::new();

		for page in doc.page_iter() {
			for annotation in doc.get_page_annotations(page).or(Err(DocParseError::AccessError(AccessTarget::Annotations)))? {			
				annotations.push((annotation).clone().try_into()?);
			}
		}

		Ok(Self {
			annotations
		})
	}
}