use crate::document::{annotation::{AnnotationType, ZAnnotation}, doc::Document};

pub fn import_annotations(doc: &Document) -> Result<Vec<ZAnnotation>, ()> {
	let mut out: Vec<ZAnnotation> = Vec::new();

	for annotation in &doc.annotations {
		if annotation.subtype != AnnotationType::Highlight { continue; }

		let z: ZAnnotation = annotation.try_into().unwrap();
		// println!("ZAnnotation: {:#?}", z);

		out.push(z);
	}

	Ok(out)
}