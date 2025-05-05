use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SelectionOutput {
	pub version: i64,
	pub selection: Selection
}

#[derive(Debug, Clone, Serialize)]
pub struct Selection {
	pub library_id: i64,
	pub document_id: i64
}